use serde::Serialize;
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

    let n = ucpack.serialize_slice(&mut buffer, &PAYLOAD).unwrap();
    let slice_serialized = &buffer[..n];

    let vec_serialized = ucpack.serialize_vec(&PAYLOAD).unwrap();

    assert_eq!(vec_serialized, slice_serialized);
    panic!("{:X?}", &buffer[..n]);
}
