// int p(long x0, long x1, long x2, long x3, long x4, long x5, long x6, long x7,
//       long x8, long x9) {
//   long z = x9;
//   return x0;
// }
// int main() {
//   long a = 1;
//   p(a, 1, 2, 3, 4, 5, 6, 7, 8, 9);
// }

int p() { return 3; }

int main() {
  long a = 1;
  long b = 2;
  p();
}