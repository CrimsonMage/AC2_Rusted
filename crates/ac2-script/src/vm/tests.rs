//! Opcode families against hand-built bytecode.

use std::sync::Arc;

use super::*;

/// An op word: opcode, tag byte, signed 16-bit immediate.
pub(super) fn w(code: u32, tag: u8, imm: i32) -> u32 {
    code | u32::from(tag) << 8 | ((imm as i16 as u16 as u32) << 16)
}

pub(super) fn vm_with(code: Vec<u32>, func_ids: Vec<Vec<u32>>, vtables: Vec<Vec<u32>>) -> Vm {
    let program = Program::from_parts(code, func_ids, vtables);
    Vm::new(Arc::new(program), Box::new(NoStates), EFuns::new())
}

/// Runs `code` from word 0 in a fresh context entered as a function.
pub(super) fn run(code: Vec<u32>) -> (Vm, Context, InterpStatus) {
    let mut vm = vm_with(code, Vec::new(), Vec::new());
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    let status = vm.execute(&mut ctx);
    (vm, ctx, status)
}

fn top(ctx: &Context, n: usize) -> Vec<(u32, u8)> {
    (ctx.sp - n..ctx.sp)
        .map(|i| (ctx.data[i], ctx.tags[i]))
        .collect()
}

const T: u32 = op::TERMINATE;

#[test]
fn integer_binops_compare_and_shift_like_x86() {
    let neg8 = -8i32 as u32;
    let cases: &[(u32, u32, u32, u32)] = &[
        (op::ADD, 7, 3, 10),
        (op::SUB, 3, 7, (-4i32) as u32),
        (op::MULT, 0x4000_0000, 4, 0),
        (op::DIV, neg8, 3, (-2i32) as u32),
        (op::MOD, neg8, 3, (-2i32) as u32),
        (op::B_LSHIFT, 1, 33, 2),
        (op::B_RSHIFT, neg8, 1, (-4i32) as u32),
        (op::B_URSHIFT, neg8, 1, 0x7FFF_FFFC),
        (op::L_SLT, neg8, 1, 1),
        (op::L_ULT, neg8, 1, 0),
        (op::L_AND, 5, 0, 0),
        (op::L_OR, 0, 9, 1),
        (op::R_EQ, 4, 4, 1),
    ];
    for &(code, a, b, expect) in cases {
        let (_, ctx, status) = run(vec![op::PUSH, a, op::PUSH, b, code, T]);
        assert_eq!(status, InterpStatus::Terminated, "{code:#x}");
        assert_eq!(top(&ctx, 1), vec![(expect, tag::INT)], "{code:#x}");
    }
    let (vm, _, status) = run(vec![w(op::PUSHV, 2, 1), w(op::PUSHV, 2, 0), op::DIV, T]);
    assert_eq!(status, InterpStatus::Crash);
    assert_eq!(vm.last_fault, Some(Fault::DivideByZero));
}

#[test]
fn pushes_carry_their_tag_and_64_bit_immediates_are_high_word_first() {
    let (_, ctx, _) = run(vec![
        w(op::PUSHV, 4, -3),
        w(op::PUSH | op::LONG, 2, 0),
        0x1122_3344,
        0x5566_7788,
        w(op::PUSHV | op::LONG, 2, -1),
        T,
    ]);
    assert_eq!(
        top(&ctx, 5),
        vec![
            ((-3i32) as u32, 4),
            (0x5566_7788, 0x82),
            (0x1122_3344, 0x82),
            (u32::MAX, 0x82),
            (u32::MAX, 0x82)
        ]
    );
}

#[test]
fn long_arithmetic_carries_and_right_shifts_are_logical() {
    let push64 = |v: u64| vec![w(0x82, 2, 0), (v >> 32) as u32, v as u32];
    let mut code = push64(0x0000_0000_FFFF_FFFF);
    code.extend(push64(1));
    code.push(op::ADD | op::LONG);
    code.push(T);
    let (_, ctx, _) = run(code);
    assert_eq!(top(&ctx, 2), vec![(0, 0x82), (1, 0x82)]);

    let mut code = push64((-16i64) as u64);
    code.extend(push64(2));
    code.push(op::B_RSHIFT | op::LONG);
    code.push(T);
    let (_, ctx, _) = run(code);
    let v = u64::from(ctx.data[ctx.sp - 2]) | u64::from(ctx.data[ctx.sp - 1]) << 32;
    assert_eq!(
        v,
        (-16i64 as u64) >> 2,
        "_aullshr even for the signed shift"
    );

    let mut code = push64(5);
    code.extend(push64(7));
    code.push(op::L_SLT | op::LONG);
    code.push(T);
    let (_, ctx, _) = run(code);
    assert_eq!(top(&ctx, 1), vec![(1, tag::INT)]);
}

#[test]
fn float_ops_round_once_and_convert_like_ftol() {
    let f = |v: f32| v.to_bits();
    let (_, ctx, _) = run(vec![op::PUSH, f(1.5), op::PUSH, f(2.25), op::FMULT, T]);
    assert_eq!(top(&ctx, 1), vec![(f(3.375), tag::FLOAT)]);
    let (_, ctx, _) = run(vec![op::PUSH, f(-2.75), op::F2I, T]);
    assert_eq!(top(&ctx, 1), vec![((-2i32) as u32, tag::INT)]);
    let (_, ctx, _) = run(vec![op::PUSH, f(3e9), op::F2I, T]);
    assert_eq!(top(&ctx, 1), vec![(0x8000_0000, tag::INT)]);
    let (_, ctx, _) = run(vec![w(op::PUSHV, 2, -5), op::I2F, T]);
    assert_eq!(top(&ctx, 1), vec![(f(-5.0), tag::FLOAT)]);
    let (_, ctx, _) = run(vec![op::PUSH, u32::MAX, op::U2F, T]);
    assert_eq!(top(&ctx, 1), vec![(f(4_294_967_296.0), tag::FLOAT)]);
    let (_, ctx, _) = run(vec![op::PUSH, f(1.0), op::PUSH, f(f32::NAN), op::L_FLTE, T]);
    assert_eq!(top(&ctx, 1), vec![(0, tag::INT)]);
    let (_, ctx, _) = run(vec![op::PUSH, f(0.0), op::F_L_NOT, T]);
    assert_eq!(top(&ctx, 1), vec![(1, tag::INT)]);
    // F2F widens to a double; I2I sign-extends; the long form of I2I narrows back.
    let (_, ctx, _) = run(vec![op::PUSH, f(0.5), op::F2F, T]);
    let bits = 0.5f64.to_bits();
    assert_eq!(
        top(&ctx, 2),
        vec![(bits as u32, 0x83), ((bits >> 32) as u32, 0x83)]
    );
    let (_, ctx, _) = run(vec![w(op::PUSHV, 2, -2), op::I2I, op::I2I | op::LONG, T]);
    assert_eq!(top(&ctx, 1), vec![((-2i32) as u32, tag::INT)]);
}

#[test]
fn double_ops_and_time_values() {
    let push_d = |v: f64| {
        vec![
            w(0x82, 3, 0),
            (v.to_bits() >> 32) as u32,
            v.to_bits() as u32,
        ]
    };
    let mut code = push_d(1.25);
    code.extend(push_d(0.5));
    code.push(op::FSUB | op::LONG);
    code.push(op::F2T | op::LONG);
    code.push(T);
    let (_, ctx, _) = run(code);
    let bits = 0.75f64.to_bits();
    assert_eq!(
        top(&ctx, 2),
        vec![(bits as u32, tag::TIME), ((bits >> 32) as u32, tag::TIME)]
    );
    let mut code = push_d(-7.9);
    code.push(op::F2I | op::LONG);
    code.push(T);
    let (_, ctx, _) = run(code);
    assert_eq!(top(&ctx, 2), vec![((-7i32) as u32, 0x82), (u32::MAX, 0x82)]);
}

#[test]
fn branches_count_bytes_from_the_next_instruction() {
    // if (0) skip one word; the skipped PUSHV never runs.
    let (_, ctx, _) = run(vec![
        w(op::PUSHV, 2, 0),
        w(op::IF, 0, 4),
        w(op::PUSHV, 2, 111),
        w(op::PUSHV, 2, 222),
        w(op::RJMP, 0, 4),
        w(op::PUSHV, 2, 333),
        T,
    ]);
    assert_eq!(top(&ctx, 1), vec![(222, tag::INT)]);
    // switch: default, count, (value, offset) pairs; offsets count from the table's end.
    for (value, expect) in [(5u32, 50u32), (9, 90), (1, 10)] {
        let (_, ctx, _) = run(vec![
            op::PUSH,
            value,
            op::SWITCH,
            16, // default: the fourth word after the table
            2,
            5,
            0,
            9,
            8,
            w(op::PUSHV, 2, 50),
            w(op::RJMP, 0, 12),
            w(op::PUSHV, 2, 90),
            w(op::RJMP, 0, 4),
            w(op::PUSHV, 2, 10),
            T,
        ]);
        assert_eq!(ctx.data[ctx.sp - 1], expect, "switch {value}");
    }
    let (vm, _, status) = run(vec![w(op::RJMP, 0, 2), T]);
    assert_eq!(
        (status, vm.last_fault),
        (InterpStatus::Crash, Some(Fault::BadJump))
    );
}

#[test]
fn frames_hold_locals_and_unknown_opcodes_crash() {
    let (_, ctx, _) = run(vec![
        w(op::PUSH_FRAME, 0, 2),
        w(op::PUSHV, 2, 9),
        w(op::PUSHV, 2, 1),
        op::STORE,
        op::POP,
        w(op::LOAD, 0, 1),
        T,
    ]);
    assert_eq!(ctx.fp, 0);
    assert_eq!(ctx.tags[0], tag::UNINIT);
    assert_eq!((ctx.data[1], ctx.tags[1]), (9, tag::INT));
    assert_eq!(top(&ctx, 1), vec![(9, tag::INT)]);
    let (vm, _, status) = run(vec![0x7F, T]);
    assert_eq!(status, InterpStatus::Crash);
    assert_eq!(vm.last_fault, Some(Fault::UnknownOpcode(0x7F)));
    // R_EQ has no 64-bit form.
    let (vm, _, _) = run(vec![op::R_EQ | op::LONG, T]);
    assert_eq!(vm.last_fault, Some(Fault::UnknownOpcode(0xC8)));
}
