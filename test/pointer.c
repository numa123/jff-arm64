#include "test.h"

int main() {
  ASSERT(3, ({
           int x1 = 3;
           *&x1;
         }));
  ASSERT(3, ({
           int x2 = 3;
           int *y2 = &x2;
           int **z2 = &y2;
           **z2;
         }));
  ASSERT(5, ({
           int x3 = 3;
           int y3 = 5;
           *(&x3 + 1);
         }));
  ASSERT(3, ({
           int x4 = 3;
           int y4 = 5;
           *(&y4 - 1);
         }));
  ASSERT(5, ({
           int x5 = 3;
           int y5 = 5;
           *(&x5 - (-1));
         }));
  ASSERT(5, ({
           int x6 = 3;
           int *y6 = &x6;
           *y6 = 5;
           x6;
         }));
  ASSERT(7, ({
           int x7 = 3;
           int y7 = 5;
           *(&x7 + 1) = 7;
           y7;
         }));
  ASSERT(7, ({
           int x8 = 3;
           int y8 = 5;
           *(&y8 - 2 + 1) = 7;
           x8;
         }));
  ASSERT(5, ({
           int x9 = 3;
           (&x9 + 2) - &x9 + 3;
         }));
  ASSERT(8, ({
           int x10, y10;
           x10 = 3;
           y10 = 5;
           x10 + y10;
         }));
  ASSERT(8, ({
           int x11 = 3, y11 = 5;
           x11 + y11;
         }));
  ASSERT(3, ({
           int x12[2];
           int *y12 = &x12;
           *y12 = 3;
           *x12;
         }));
  ASSERT(3, ({
           int x13[3];
           *x13 = 3;
           *(x13 + 1) = 4;
           *(x13 + 2) = 5;
           *x13;
         }));
  ASSERT(4, ({
           int x14[3];
           *x14 = 3;
           *(x14 + 1) = 4;
           *(x14 + 2) = 5;
           *(x14 + 1);
         }));
  ASSERT(5, ({
           int x15[3];
           *x15 = 3;
           *(x15 + 1) = 4;
           *(x15 + 2) = 5;
           *(x15 + 2);
         }));
  ASSERT(0, ({
           int x16[2][3];
           int *y16 = x16;
           *y16 = 0;
           **x16;
         }));
  ASSERT(1, ({
           int x17[2][3];
           int *y17 = x17;
           *(y17 + 1) = 1;
           *(*x17 + 1);
         }));
  ASSERT(2, ({
           int x18[2][3];
           int *y18 = x18;
           *(y18 + 2) = 2;
           *(*x18 + 2);
         }));
  ASSERT(3, ({
           int x19[2][3];
           int *y19 = x19;
           *(y19 + 3) = 3;
           **(x19 + 1);
         }));
  ASSERT(4, ({
           int x20[2][3];
           int *y20 = x20;
           *(y20 + 4) = 4;
           *(*(x20 + 1) + 1);
         }));
  ASSERT(5, ({
           int x21[2][3];
           int *y21 = x21;
           *(y21 + 5) = 5;
           *(*(x21 + 1) + 2);
         }));
  ASSERT(3, ({
           int x22[3];
           *x22 = 3;
           x22[1] = 4;
           x22[2] = 5;
           *x22;
         }));
  ASSERT(4, ({
           int x23[3];
           *x23 = 3;
           x23[1] = 4;
           x23[2] = 5;
           *(x23 + 1);
         }));
  ASSERT(5, ({
           int x24[3];
           *x24 = 3;
           x24[1] = 4;
           x24[2] = 5;
           *(x24 + 2);
         }));
  ASSERT(5, ({
           int x25[3];
           *x25 = 3;
           x25[1] = 4;
           2 [x25] = 5;
           *(x25 + 2);
         }));
  ASSERT(0, ({
           int x26[2][3];
           int *y26 = x26;
           y26[0] = 0;
           x26[0][0];
         }));
  ASSERT(1, ({
           int x27[2][3];
           int *y27 = x27;
           y27[1] = 1;
           x27[0][1];
         }));
  ASSERT(2, ({
           int x28[2][3];
           int *y28 = x28;
           y28[2] = 2;
           x28[0][2];
         }));
  ASSERT(3, ({
           int x29[2][3];
           int *y29 = x29;
           y29[3] = 3;
           x29[1][0];
         }));
  ASSERT(4, ({
           int x30[2][3];
           int *y30 = x30;
           y30[4] = 4;
           x30[1][1];
         }));
  ASSERT(5, ({
           int x31[2][3];
           int *y31 = x31;
           y31[5] = 5;
           x31[1][2];
         }));
  printf("OK\n");
  return 0;
}
