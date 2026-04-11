use super::*;
use std::collections::BTreeMap;

#[tokio::test]
#[serial]
async fn test_f32_roundtrip() {
    let ctx = TestContext::new().await;

    ctx.client.set_price_f32(0.0).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f32, 0.0);

    ctx.client.set_price_f32(1.5).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f32, 1.5);

    ctx.client.set_price_f32(-0.25).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f32, -0.25);

    ctx.client.set_price_f32(f32::MIN).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f32, f32::MIN);

    ctx.client.set_price_f32(f32::MAX).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f32, f32::MAX);

    ctx.client.set_price_f32(f32::EPSILON).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f32, f32::EPSILON);
}

#[tokio::test]
#[serial]
async fn test_f64_roundtrip() {
    let ctx = TestContext::new().await;

    ctx.client.set_price_f64(0.0).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f64, 0.0);

    ctx.client.set_price_f64(std::f64::consts::PI).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f64, std::f64::consts::PI);

    ctx.client.set_price_f64(-1e300).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f64, -1e300);

    ctx.client.set_price_f64(f64::MIN).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f64, f64::MIN);

    ctx.client.set_price_f64(f64::MAX).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f64, f64::MAX);
}

#[tokio::test]
#[serial]
async fn test_float_infinities() {
    let ctx = TestContext::new().await;

    ctx.client.set_price_f32(f32::INFINITY).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f32, f32::INFINITY);

    ctx.client.set_price_f32(f32::NEG_INFINITY).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f32, f32::NEG_INFINITY);

    ctx.client.set_price_f64(f64::INFINITY).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f64, f64::INFINITY);

    ctx.client.set_price_f64(f64::NEG_INFINITY).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.price_f64, f64::NEG_INFINITY);
}

#[tokio::test]
#[serial]
async fn test_btreeset_string() {
    let ctx = TestContext::new().await;
    assert!(ctx.client.state().await.tag_set.is_empty());

    ctx.client.insert_tag("banana").await.await_confirmation().await;
    ctx.client.insert_tag("apple").await.await_confirmation().await;
    ctx.client.insert_tag("cherry").await.await_confirmation().await;
    ctx.client.insert_tag("apple").await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.tag_set.len(), 3);
    assert!(state.tag_set.contains("apple"));
    assert!(state.tag_set.contains("banana"));
    assert!(state.tag_set.contains("cherry"));

    let ordered: Vec<&String> = state.tag_set.iter().collect();
    assert_eq!(ordered, vec!["apple", "banana", "cherry"]);

    ctx.client.remove_tag("banana").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.tag_set.len(), 2);
    assert!(!state.tag_set.contains("banana"));
}

#[tokio::test]
#[serial]
async fn test_btreeset_u64_ordered() {
    let ctx = TestContext::new().await;

    for v in [42u64, 7, 1000, 3, 7, 999] {
        ctx.client.insert_int(v).await.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    let ordered: Vec<u64> = state.int_set.iter().copied().collect();
    assert_eq!(ordered, vec![3, 7, 42, 999, 1000]);

    ctx.client.remove_int(7).await.await_confirmation().await;
    let state = ctx.client.state().await;
    let ordered: Vec<u64> = state.int_set.iter().copied().collect();
    assert_eq!(ordered, vec![3, 42, 999, 1000]);
}

#[tokio::test]
#[serial]
async fn test_bool_roundtrip() {
    let ctx = TestContext::new().await;
    assert!(!ctx.client.state().await.primitives.flag_bool);

    ctx.client.set_bool(true).await.await_confirmation().await;
    assert!(ctx.client.state().await.primitives.flag_bool);

    ctx.client.set_bool(false).await.await_confirmation().await;
    assert!(!ctx.client.state().await.primitives.flag_bool);
}

#[tokio::test]
#[serial]
async fn test_u8_boundaries() {
    let ctx = TestContext::new().await;

    ctx.client.set_u8(0).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.small_u, 0);

    ctx.client.set_u8(127).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.small_u, 127);

    ctx.client.set_u8(u8::MAX).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.small_u, u8::MAX);
}

#[tokio::test]
#[serial]
async fn test_u16_boundaries() {
    let ctx = TestContext::new().await;

    ctx.client.set_u16(0).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.mid_u, 0);

    ctx.client.set_u16(u16::MAX).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.mid_u, u16::MAX);
}

#[tokio::test]
#[serial]
async fn test_u128_boundaries() {
    let ctx = TestContext::new().await;

    ctx.client.set_u128(0).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.big_u, 0);

    ctx.client.set_u128(u128::MAX).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.big_u, u128::MAX);

    let large: u128 = (u64::MAX as u128) + 1;
    ctx.client.set_u128(large).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.big_u, large);
}

#[tokio::test]
#[serial]
async fn test_i8_boundaries() {
    let ctx = TestContext::new().await;

    ctx.client.set_i8(0).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s8, 0);

    ctx.client.set_i8(i8::MIN).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s8, i8::MIN);

    ctx.client.set_i8(i8::MAX).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s8, i8::MAX);

    ctx.client.set_i8(-1).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s8, -1);
}

#[tokio::test]
#[serial]
async fn test_i16_boundaries() {
    let ctx = TestContext::new().await;

    ctx.client.set_i16(i16::MIN).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s16, i16::MIN);

    ctx.client.set_i16(i16::MAX).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s16, i16::MAX);
}

#[tokio::test]
#[serial]
async fn test_i32_boundaries() {
    let ctx = TestContext::new().await;

    ctx.client.set_i32(i32::MIN).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s32, i32::MIN);

    ctx.client.set_i32(i32::MAX).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s32, i32::MAX);
}

#[tokio::test]
#[serial]
async fn test_i64_boundaries() {
    let ctx = TestContext::new().await;

    ctx.client.set_i64(i64::MIN).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s64, i64::MIN);

    ctx.client.set_i64(i64::MAX).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s64, i64::MAX);
}

#[tokio::test]
#[serial]
async fn test_i128_boundaries() {
    let ctx = TestContext::new().await;

    ctx.client.set_i128(i128::MIN).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s128, i128::MIN);

    ctx.client.set_i128(i128::MAX).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.s128, i128::MAX);
}

#[tokio::test]
#[serial]
async fn test_option_string_roundtrip() {
    let ctx = TestContext::new().await;
    assert!(ctx.client.state().await.primitives.maybe_string.is_none());

    ctx.client.set_maybe_string(Some("hello".to_string())).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.maybe_string, Some("hello".to_string()));

    ctx.client.set_maybe_string(Some(String::new())).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.maybe_string, Some(String::new()));

    ctx.client.set_maybe_string(None).await.await_confirmation().await;
    assert!(ctx.client.state().await.primitives.maybe_string.is_none());
}

#[tokio::test]
#[serial]
async fn test_option_int_roundtrip() {
    let ctx = TestContext::new().await;
    assert!(ctx.client.state().await.primitives.maybe_int.is_none());

    ctx.client.set_maybe_int(Some(0)).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.maybe_int, Some(0));

    ctx.client.set_maybe_int(Some(u64::MAX)).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.maybe_int, Some(u64::MAX));

    ctx.client.set_maybe_int(None).await.await_confirmation().await;
    assert!(ctx.client.state().await.primitives.maybe_int.is_none());
}

#[tokio::test]
#[serial]
async fn test_vec_u32_roundtrip() {
    let ctx = TestContext::new().await;
    assert!(ctx.client.state().await.primitives.numbers.is_empty());

    for v in [1u32, 2, 3, u32::MAX, 0] {
        ctx.client.push_number(v).await.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    assert_eq!(state.primitives.numbers, vec![1, 2, 3, u32::MAX, 0]);

    ctx.client.clear_numbers().await.await_confirmation().await;
    assert!(ctx.client.state().await.primitives.numbers.is_empty());
}

#[tokio::test]
#[serial]
async fn test_fixed_array_roundtrip() {
    let ctx = TestContext::new().await;
    assert_eq!(ctx.client.state().await.primitives.fixed_bytes, [0u8; 4]);

    ctx.client.set_fixed_bytes([1, 2, 3, 4]).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.fixed_bytes, [1, 2, 3, 4]);

    ctx.client.set_fixed_bytes([0xFF; 4]).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.fixed_bytes, [0xFF; 4]);
}

#[tokio::test]
#[serial]
async fn test_tuple_roundtrip() {
    let ctx = TestContext::new().await;
    assert_eq!(ctx.client.state().await.primitives.pair, (0u64, String::new()));

    ctx.client.set_pair(42, "answer").await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.pair, (42u64, "answer".to_string()));

    ctx.client.set_pair(u64::MAX, "").await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.primitives.pair, (u64::MAX, String::new()));
}

#[tokio::test]
#[serial]
async fn test_btreemap_roundtrip() {
    let ctx = TestContext::new().await;
    assert!(ctx.client.state().await.primitives.btree.is_empty());

    ctx.client.btree_insert("zebra", 1).await.await_confirmation().await;
    ctx.client.btree_insert("alpha", 2).await.await_confirmation().await;
    ctx.client.btree_insert("mango", 3).await.await_confirmation().await;
    ctx.client.btree_insert("alpha", 99).await.await_confirmation().await;

    let state = ctx.client.state().await;
    let expected: BTreeMap<String, u64> =
        [("alpha".to_string(), 99), ("mango".to_string(), 3), ("zebra".to_string(), 1)]
            .into_iter()
            .collect();
    assert_eq!(state.primitives.btree, expected);

    let keys_in_order: Vec<&String> = state.primitives.btree.keys().collect();
    assert_eq!(keys_in_order, vec!["alpha", "mango", "zebra"]);

    ctx.client.btree_remove("mango").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.primitives.btree.len(), 2);
    assert!(!state.primitives.btree.contains_key("mango"));
}
