#include "test.h"
int main() {
  ASSERT(0, 0);
  ASSERT(42, 42);
  ASSERT(21, 5 + 20 - 4);
  ASSERT(41, 12 + 34 - 5);
  ASSERT(47, 5 + 6 * 7);
  ASSERT(15, 5 * (9 - 6));
  ASSERT(4, (3 + 5) / 2);
  ASSERT(0, 15 % 5);
  ASSERT(3, 15 % 4);
  ASSERT(10, -10 + 20);
  ASSERT(10, - -10);
  ASSERT(10, - -+10);
  ASSERT(0, 0 == 1);
  ASSERT(1, 42 == 42);
  ASSERT(1, 0 != 1);
  ASSERT(0, 42 != 42);
  ASSERT(1, 0 < 1);
  ASSERT(0, 1 < 1);
  ASSERT(0, 2 < 1);
  ASSERT(1, 0 <= 1);
  ASSERT(1, 1 <= 1);
  ASSERT(0, 2 <= 1);
  ASSERT(1, 1 > 0);
  ASSERT(0, 1 > 1);
  ASSERT(0, 1 > 2);
  ASSERT(1, 1 >= 0);
  ASSERT(1, 1 >= 1);
  ASSERT(0, 1 >= 2);
  ASSERT(1, 5 && 2);
  ASSERT(1, 1 && 1);
  ASSERT(0, 0 && 1);
  ASSERT(1, 0 || 2);
  ASSERT(0, 0 || 0);
  ASSERT(0, 1 != 1 || 0 == 0 && 1 == 0);
  ASSERT(1, 1 == 1 && 0 != 0 || 1 != 0);
  ASSERT(0, 1 & 0);
  ASSERT(3, 2 | 1);
  ASSERT(6, 7 ^ 1);

  ASSERT(7, ({
           int i = 2;
           i += 5;
           i;
         }));
  ASSERT(7, ({
           int i = 2;
           i += 5;
         }));
  ASSERT(3, ({
           int i = 5;
           i -= 2;
           i;
         }));
  ASSERT(3, ({
           int i = 5;
           i -= 2;
         }));
  ASSERT(6, ({
           int i = 3;
           i *= 2;
           i;
         }));
  ASSERT(6, ({
           int i = 3;
           i *= 2;
         }));
  ASSERT(3, ({
           int i = 6;
           i /= 2;
           i;
         }));
  ASSERT(3, ({
           int i = 6;
           i /= 2;
         }));

  ASSERT(0, 0 & 1);
  ASSERT(1, 3 & 1);
  ASSERT(3, 7 & 3);
  ASSERT(10, -1 & 10);
  ASSERT(1, 0 | 1);
  ASSERT(0, 0 ^ 0);
  // ASSERT(0, 0b1111 ^ 0b1111);
  // ASSERT(0b110100, 0b111000 ^ 0b001100);
  ASSERT(2, ({
           int i = 6;
           i &= 3;
           i;
         }));
  ASSERT(7, ({
           int i = 6;
           i |= 3;
           i;
         }));
  ASSERT(10, ({
           int i = 15;
           i ^= 5;
           i;
         }));

  // ASSERT(0, 1073741824 * 100 / 100);
  printf("OK\n");
  return 0;
}