; ModuleID = 'probe4.acddb030b9b34784-cgu.0'
source_filename = "probe4.acddb030b9b34784-cgu.0"
target datalayout = "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-windows-msvc"

@alloc_ba945917280f9c391d5ee6d45183fc34 = private unnamed_addr constant <{ [75 x i8] }> <{ [75 x i8] c"/rustc/8142a319ed5c1d1f96e5a1881a6546e463b77c8f\\library\\core\\src\\num\\mod.rs" }>, align 1
@alloc_210b30898eb66ff8e7b5985570d65580 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_ba945917280f9c391d5ee6d45183fc34, [16 x i8] c"K\00\00\00\00\00\00\00w\04\00\00\05\00\00\00" }>, align 8
@str.0 = internal constant [25 x i8] c"attempt to divide by zero"

; probe4::probe
; Function Attrs: uwtable
define void @_ZN6probe45probe17haf81f94746672a01E() unnamed_addr #0 {
start:
  %0 = call i1 @llvm.expect.i1(i1 false, i1 false)
  br i1 %0, label %panic.i, label %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h3ab175dcc00af49fE.exit"

panic.i:                                          ; preds = %start
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17h92a4243d26e449a8E(ptr align 1 @str.0, i64 25, ptr align 8 @alloc_210b30898eb66ff8e7b5985570d65580) #3
  unreachable

"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h3ab175dcc00af49fE.exit": ; preds = %start
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(none)
declare i1 @llvm.expect.i1(i1, i1) #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking5panic17h92a4243d26e449a8E(ptr align 1, i64, ptr align 8) unnamed_addr #2

attributes #0 = { uwtable "target-cpu"="x86-64" }
attributes #1 = { nocallback nofree nosync nounwind willreturn memory(none) }
attributes #2 = { cold noinline noreturn uwtable "target-cpu"="x86-64" }
attributes #3 = { noreturn }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.74.0-nightly (8142a319e 2023-09-13)"}