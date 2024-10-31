use serde::{Deserialize, Serialize};
use ucpack::UcPack;

#[test]
fn test_continuity() {
    let ucpack = UcPack::default();
    let mut buffer = vec![0; 1000];

    #[derive(Serialize)]
    struct TestPayload {
        a: u16,
        b: u8,
        c: f32,
    }
    const PAYLOAD: TestPayload = TestPayload { a: 1, b: 2, c: 1.0 };

    let n = ucpack.serialize_slice(&PAYLOAD, &mut buffer).unwrap();
    let slice_serialized = &buffer[..n];

    let vec_serialized = ucpack.serialize_vec(&PAYLOAD).unwrap();

    assert_eq!(vec_serialized, slice_serialized);
}

#[test]
fn test_serialize_deserialize() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum TestEnum {
        Tag1,
        Tag2(u16),
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestPayload {
        a: u16,
        b: u8,
        c: f32,
        d: TestEnum,
    }

    const PAYLOAD: TestPayload = TestPayload {
        a: 1,
        b: 2,
        c: 1.0,
        d: TestEnum::Tag2(10),
    };

    let ucpack = UcPack::default();
    let serialized = ucpack.serialize_vec(&PAYLOAD).unwrap();
    let deserialized: TestPayload = ucpack.deserialize_slice(&serialized).unwrap();

    assert_eq!(PAYLOAD, deserialized);
}

// #[test]
// fn test_enum() {
//     #[derive(Serialize)]
//     enum Test {
//         Command2(u8),
//         Command1 { a: u16, b: u8 },
//     }

//     let ucpack = UcPack::default();

//     let res = ucpack.serialize_vec(&Test::Command2(1)).unwrap();
//     panic!("{res:?}");
// }
