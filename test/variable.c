#include "test.h"
int g1, g2[4];
int main() {
  ASSERT(3, ({
           int a1;
           a1 = 3;
           a1;
         }));
  ASSERT(3, ({
           int a2 = 3;
           a2;
         }));
  ASSERT(8, ({
           int a3 = 3;
           int z3 = 5;
           a3 + z3;
         }));
  ASSERT(3, ({
           int a4 = 3;
           a4;
         }));
  ASSERT(8, ({
           int a5 = 3;
           int z5 = 5;
           a5 + z5;
         }));
  ASSERT(6, ({
           int a6;
           int b6;
           a6 = b6 = 3;
           a6 + b6;
         }));
  ASSERT(3, ({
           int foo7 = 3;
           foo7;
         }));
  ASSERT(8, ({
           int foo123 = 3;
           int bar8 = 5;
           foo123 + bar8;
         }));
  ASSERT(8, ({
           int x9;
           sizeof(x9);
         }));
  ASSERT(8, ({
           int x10;
           sizeof x10;
         }));
  ASSERT(8, ({
           int *x11;
           sizeof(x11);
         }));
  ASSERT(32, ({
           int x12[4];
           sizeof(x12);
         }));
  ASSERT(96, ({
           int x13[3][4];
           sizeof(x13);
         }));
  ASSERT(32, ({
           int x14[3][4];
           sizeof(*x14);
         }));
  ASSERT(8, ({
           int x15[3][4];
           sizeof(**x15);
         }));
  ASSERT(9, ({
           int x16[3][4];
           sizeof(**x16) + 1;
         }));
  ASSERT(9, ({
           int x17[3][4];
           sizeof **x17 + 1;
         }));
  ASSERT(8, ({
           int x18[3][4];
           sizeof(**x18 + 1);
         }));
  ASSERT(8, ({
           int x19 = 1;
           sizeof(x19 = 2);
         }));
  ASSERT(1, ({
           int x20 = 1;
           sizeof(x20 = 2);
           x20;
         }));
  ASSERT(0, g1);
  ASSERT(3, ({
           g1 = 3;
           g1;
         }));
  ASSERT(0, ({
           g2[0] = 0;
           g2[1] = 1;
           g2[2] = 2;
           g2[3] = 3;
           g2[0];
         }));
  ASSERT(1, ({
           g2[0] = 0;
           g2[1] = 1;
           g2[2] = 2;
           g2[3] = 3;
           g2[1];
         }));
  ASSERT(2, ({
           g2[0] = 0;
           g2[1] = 1;
           g2[2] = 2;
           g2[3] = 3;
           g2[2];
         }));
  ASSERT(3, ({
           g2[0] = 0;
           g2[1] = 1;
           g2[2] = 2;
           g2[3] = 3;
           g2[3];
         }));
  ASSERT(8, sizeof(g1));
  ASSERT(32, sizeof(g2));
  ASSERT(1, ({
           char x21 = 1;
           x21;
         }));
  // 通ったり通らなかったり。。。

  // ASSERT(1, ({
  //          char x22 = 1;
  //          char y22 = 2;
  //          x22;
  //        }));
  // なぜこれが513になるんだ
  ASSERT(2, ({
           char x23 = 1;
           char y23 = 2;
           y23;
         }));
  ASSERT(1, ({
           char x24;
           sizeof(x24);
         }));
  ASSERT(10, ({
           char x25[10];
           sizeof(x25);
         }));
  ASSERT(2, ({
           int x26 = 2;
           { int x27 = 3; }
           x26;
         }));
  ASSERT(2, ({
           int x28 = 2;
           { int x29 = 3; }
           int y28 = 4;
           x28;
         }));
  ASSERT(3, ({
           int x30 = 2;
           { x30 = 3; }
           x30;
         }));
  printf("OK\n");
  return 0;
}
