use crate::{
    cmd::{extract_args, validate_command, CommandError, CommandExecutor},
    Backend, RespArray, RespFrame, RespNull,
};
#[derive(Debug)]
pub struct HMGet {
    key: String,
    fields: Vec<String>,
}

impl CommandExecutor for HMGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        let hmap = backend.hgetall(&self.key);
        match hmap {
            Some(hmap) => {
                let mut data = Vec::with_capacity(self.fields.len());
                for field in self.fields.iter() {
                    let value = hmap.get(field);
                    match value {
                        Some(value) => data.push(value.clone()),
                        None => data.push(RespNull.into()),
                    }
                }
                RespArray::new(data).into()
            }
            None => RespArray::new([]).into(),
        }
    }
}

impl TryFrom<RespArray> for HMGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hmget"], usize::MAX)?;
        let args = extract_args(value, 1)?.into_iter();
        let mut data = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                RespFrame::BulkString(s) => {
                    let s = String::from_utf8(s.0)?;
                    data.push(s);
                }
                _ => {
                    return Err(CommandError::InvalidArgument(
                        "Invalid key or field".to_string(),
                    ));
                }
            }
        }
        Ok(HMGet {
            key: data.remove(0),
            fields: data,
        })
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::cmd::{HSet, RESP_OK};
//     use crate::resp::BulkString;

//     #[test]
//     fn test_hmget_from_resp_array() -> anyhow::Result<()> {
//         let backend = Backend::new();
//         let cmd = HSet {
//             key: "myhash".to_string(),
//             field: "field1".to_string(),
//             value: RespFrame::BulkString(b"hello".into()),
//         };
//         let result = cmd.execute(&backend);
//         assert_eq!(result, RESP_OK.clone());

//         let cmd = HSet {
//             key: "myhash".to_string(),
//             field: "field2".to_string(),
//             value: RespFrame::BulkString(b"world".into()),
//         };
//         let result = cmd.execute(&backend);
//         assert_eq!(result, RESP_OK.clone());

//         let cmd = HMGet::try_from(RespArray::new(vec![
//             RespFrame::BulkString("HMGET".into()),
//             RespFrame::BulkString("myhash".into()),
//             RespFrame::BulkString("field1".into()),
//             RespFrame::BulkString("field2".into()),
//             RespFrame::BulkString("nofield".into()),
//         ]))?;
//         let result = cmd.execute(&backend);
//         let expected = RespArray::new(vec![
//             BulkString::from("hello").into(),
//             BulkString::from("world").into(),
//             RespNull.into(),
//         ]);
//         assert_eq!(result, expected.into());
//         Ok(())
//     }
// }
