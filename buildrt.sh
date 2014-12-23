#!/bin/sh

[ -z "$CC" ] && CC=cc
[ -z "$LD" ] && LD=cc

# I did not manage to pass the target triple ($TARGET) to cc.
# gnu99 is selected for declarations in for loop heads and MAP_ANONYMOUS.

$CC -std=gnu99 src/rt.c -c -o "$OUT_DIR"/libturingrt.o -O"$OPT_LEVEL" -fPIC

$LD -shared "$OUT_DIR"/libturingrt.o -o "$OUT_DIR"/libturingrt.so
ln -s "$OUT_DIR"/libturingrt.so "$OUT_DIR"/libturingrt.so.0

ar crus "$OUT_DIR"/libturingrt.a "$OUT_DIR"/libturingrt.o
