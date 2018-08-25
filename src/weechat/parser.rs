use nom::{be_u8,be_u16,be_i32,be_u24,be_u32,be_f64,anychar,IResult,Needed};
use std::str;

/*
 * Header included with every weechat message.
 */
#[derive(Debug, PartialEq)]
pub struct WeechatHeader {
    pub body_length: u32,
    pub compressed: bool,
}

named!(pub parse_header<WeechatHeader>,
  do_parse!(
      body_length: be_u32 >>
      compressed: be_u8   >>
          (WeechatHeader{
              body_length: body_length,
              compressed: compressed as u8 == 1,
          })
  )
);


/*
 * Different types of identifiers that Weechat Can send.
 * https://weechat.org/files/doc/stable/weechat_relay_protocol.en.html#messages
 */
#[derive(Debug, PartialEq)]
pub enum WeechatId<'a> {
    ID(&'a str)
    // BufferOpened,
    // BufferMoved,
    // BufferMerged,
    // BufferUnMerged,
    // BufferHidden,
    // BufferUnHidden,
    // BufferRenamed,
    // BufferTitleChanged,
    // BufferCleared,
    // BufferTypeChanged,
    // BufferLocalvarAdded,
    // BufferLocalvarRemoved,
    // BufferLineAdded,
    // BufferClosing,
    // Nicklist,
    // NicklistDiff,
    // Pong,
    // Upgrade,
    // UpgradeEnded
}

named!(pub parse_identifier<WeechatId>,
       do_parse!(
           strlen: be_u32 >>
           strval: take_str!(strlen) >>
               (WeechatId::ID(strval))
       )
);


/*
 * All the different types of objects that Weechat relay protocol returns.
 * https://weechat.org/files/doc/stable/weechat_relay_protocol.en.html#objects
 */
#[derive(Debug, PartialEq)]
pub enum WeechatObjects<'a> {
    WString(&'a str),
    WChar(char),
    WInt(i32),
    WLong(&'a str),
    WBuffer(&'a str),
    WPointer(&'a str),
    WTime(&'a str),
    WHashTable,
    WHdata(&'a str, HdataKeys<'a>, u32, Vec<(Vec<WeechatObjects<'a>>, Vec<WeechatObjects<'a>>)>),
    WInfo(&'a str, &'a str),
    WInfoList(&'a str, u32, Vec<InfolistItems<'a>>),
    WArray(&'a str, u32, Vec<WeechatObjects<'a>>),
}


named!(parse_bare_string<&str>,
       do_parse!(
           len: be_u32 >>
           val: take_str!(len) >>
       (val))
);

#[derive(Debug, PartialEq)]
struct HdataObjects<'a> {
    keys: HdataKeys<'a>,
}

#[derive(Debug, PartialEq)]
struct HdataKeys<'a> {
    keys: Vec<Vec<&'a str>>,
}

impl<'a> HdataKeys<'a> {
    pub fn from_raw_values(name_types: &str) -> HdataKeys {
        let mut keys = Vec::new();

        for values in name_types.split(",") {
            keys.push(values.split(":").collect::<Vec<&str>>());
        }

        HdataKeys{ keys: keys }
    }
}

named!(parse_hdata_keys<HdataKeys>,
       do_parse!(
           name: call!(parse_bare_string) >>
       (HdataKeys::from_raw_values(name)))
);



fn parse_objects_fn<'a>(input: &'a [u8], hdata_keys: &HdataKeys, hpath: &str, pointers: &Vec<WeechatObjects>)
                     -> IResult<&'a [u8], Vec<WeechatObjects<'a>>> {
    let mut objs = Vec::new();

    let mut points = Vec::new();
    points.push(input);

    println!("hpath is: {:?}", hpath);
    println!("keys are: {:?}", hdata_keys);
    println!("Pointers are: {:?}", pointers);


    for key in hdata_keys.keys.iter() {
        let parse_fn = match key[1] {
            "int" => {println!("Found int"); parse_int},
            "chr" => {println!("Found chr"); parse_chr},
            "str" => {println!("Found str"); parse_str},
            "ptr" => {println!("Found ptr"); parse_pointer},
            "lng" => {println!("Found long"); parse_long},
            "buf" => {println!("Found buf"); parse_buf},
            "arr" => {println!("Found arr"); parse_arr},
            "tim" => {println!("Found time"); parse_time},
            _     => panic!("Unexpected datatype."),
        };

        let ptr = match points.pop() {
            Some(res) => res,
            None => panic!("Error popping from vector"),
        };

        match parse_fn(ptr) {
            Ok((remaining, val)) => {
                points.push(remaining);
                objs.push(val);
                ()
            },
            Err(err) => panic!("Error parsing: {:?}", err),
        }
    }
    let ptr = match points.pop() {
        Some(res) => res,
        None => panic!("Error popping from vector."),
    };

    Ok((ptr, objs))
}


named!(pub parse_hdata<WeechatObjects>,
       do_parse!(
           h_path: call!(parse_bare_string) >>
           keys: call!(parse_hdata_keys) >>
           count: be_u32 >>
           objects: count!(
               do_parse!(
                   pointers: count!(parse_pointer, h_path.matches("/").count() + 1) >>
                   objects: call!(parse_objects_fn, &keys, h_path, &pointers) >>
               (pointers, objects)
               )
               , count as usize) >>
        (WeechatObjects::WHdata(h_path, keys, count, objects))
       )
);


// named!(pub parse_hashtable<WeechatObjects>,
//        do_parse!(
//            key_type: call!(parse_type) >>
//            val_type: call!(val_type) >>
//            count: be_u32 >>
//        ()
//        )
// )


named!(pub parse_time<WeechatObjects>,
       do_parse!(
           strlen: be_u8 >>
           strval: take_str!(strlen) >>
       (WeechatObjects::WTime(strval))
       )
);

named!(pub parse_info<WeechatObjects>,
       do_parse!(
           namelen: be_u32 >>
           name: take_str!(namelen) >>
           versionlen: be_u32 >>
           version: take_str!(versionlen) >>
       (WeechatObjects::WInfo(name, version)))
);


#[derive(Debug, PartialEq)]
struct InfolistItem<'a> {
    name: &'a str,
    value: WeechatObjects<'a>
}

named!(parse_infolist_item<InfolistItem>,
       do_parse!(
           namelen: be_u32 >>
           name: take_str!(namelen) >>
           value: switch!(take_str!(3),
                   "int" => call!(parse_int)    |
                   "chr" => call!(parse_chr)    |
                   "str" => call!(parse_str)    |
                   "ptr" => call!(parse_pointer)|
                   "lng" => call!(parse_long)   |
                   "buf" => call!(parse_buf)    |
                   "arr" => call!(parse_arr)
           ) >>
       (InfolistItem{name: name, value: value}))
);

#[derive(Debug, PartialEq)]
struct InfolistItems<'a> {
    count: u32,
    objects: Vec<InfolistItem<'a>>
}


named!(parse_infolist_items<InfolistItems>,
       do_parse!(
           count: be_u32 >>
           objects: count!(call!(parse_infolist_item), count as usize) >>
       (InfolistItems{count: count, objects:objects}))
);


named!(pub parse_infolist<WeechatObjects>,
       do_parse!(
           strlen: be_u32 >>
           name: take_str!(strlen) >>
           count: be_u32 >>
           items: count!(call!(parse_infolist_items), count as usize) >>
       (WeechatObjects::WInfoList(name, count, items))
       )
);

named!(pub parse_type<WeechatObjects>,
       do_parse!(
           val: take_str!(3) >>
       (WeechatObjects::WString(val)))
);

named!(pub parse_chr<WeechatObjects>,
       do_parse!(
           val: anychar >>
       (WeechatObjects::WChar(val))
       )
);

named!(pub parse_int<WeechatObjects>,
       do_parse!(
           val: be_i32 >>
       (WeechatObjects::WInt(val))
       )
);

named!(pub parse_str<WeechatObjects>,
       do_parse!(
           strlen: be_u32 >>
           strval: take_str!(strlen) >>
       (WeechatObjects::WString(strval)))
);

named!(pub parse_long<WeechatObjects>,
       do_parse!(
           strlen: be_u32 >>
           strval: take_str!(strlen) >>
       (WeechatObjects::WLong(strval)))
);

named!(pub parse_buf<WeechatObjects>,
      do_parse!(
           strlen: be_u32 >>
           strval: take_str!(strlen) >>
       (WeechatObjects::WBuffer(strval)))
);

named!(pub parse_pointer<WeechatObjects>,
       do_parse!(
           strlen: be_u8 >>
           strval: take_str!(strlen) >>
       (WeechatObjects::WPointer(strval)))
);


fn parse_arr_objects<'a>(input: &'a [u8], objtype: &str, count: u32) -> IResult<&'a [u8], Vec<WeechatObjects<'a>>> {
    let mut objs = Vec::new();

    let parse_fn = match objtype {
        "int" => {println!("Found int"); parse_int},
        "lon" => {println!("Found long"); parse_long},
        "chr" => {println!("Found chr"); parse_chr},
        "str" => {println!("Found str"); parse_str},
        "ptr" => {println!("Found ptr"); parse_pointer},
        "buf" => {println!("Found buf"); parse_buf},
        "arr" => {println!("Found arr"); parse_arr},
        "tim" => {println!("Found time"); parse_time},
        "htb" => panic!("Unplemented type hash table"),
        "hda" => panic!("Unimplemented type hda"),
        "inf" => panic!("Unimplemented type inf"),
        "inl" => panic!("Unplemented type inl"),
        _     => parse_str,
    };

    for each in [0..count].iter() {
        let result = parse_fn(input)?;
        objs.push(result.1);
        let input = result.0;
    }

    Ok((input, objs))

}

named!(pub parse_arr<WeechatObjects>,
       dbg_dmp!( do_parse!(
           objtype: take_str!(3) >>
           objcount: be_u32 >>
           objects:  apply!(parse_arr_objects, objtype, objcount) >>
           (WeechatObjects::WArray(objtype, objcount, objects))
       ))
);


// fn parse_objects(input: &[u8]) -> IResult<&[u8], &[u8]> {
//     let result = match parse_type(input) {
//         Ok(res) => res,
//         Err(err) => panic!("Error parsing object type."),
//     };

//     match result.1 {
//         "chr" => parse_chr(result.0),
//         "int" => parse_int(result.0),
//         "hda" => {
//             parse_hdata(result.0)
//         },
//     }
// }
