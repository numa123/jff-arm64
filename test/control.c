#include "test.h"
int main() {
  ASSERT(3, ({
           int x1;
           if (0)
             x1 = 2;
           else
             x1 = 3;
           x1;
         }));
  ASSERT(3, ({
           int x2;
           if (1 - 1)
             x2 = 2;
           else
             x2 = 3;
           x2;
         }));
  ASSERT(2, ({
           int x3;
           if (1)
             x3 = 2;
           else
             x3 = 3;
           x3;
         }));
  ASSERT(2, ({
           int x4;
           if (2 - 1)
             x4 = 2;
           else
             x4 = 3;
           x4;
         }));
  ASSERT(55, ({
           int i1 = 0;
           int j1 = 0;
           for (i1 = 0; i1 <= 10; i1 = i1 + 1)
             j1 = i1 + j1;
           j1;
         }));
  ASSERT(10, ({
           int i2 = 0;
           while (i2 < 10)
             i2 = i2 + 1;
           i2;
         }));
  ASSERT(3, ({
           1;
           { 2; }
           3;
         }));
  ASSERT(5, ({
           ;
           ;
           ;
           5;
         }));
  ASSERT(55, ({
           int i3 = 0;
           int j3 = 0;
           while (i3 <= 10) {
             j3 = i3 + j3;
             i3 = i3 + 1;
           }
           j3;
         }));
  printf("OK\n");
  return 0;
}