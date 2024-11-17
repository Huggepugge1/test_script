#include <stdio.h>
#include <stdlib.h>

int main() {
    char *a = malloc(sizeof(char) * 100);
    while (1) {
        fgets(a, 100, stdin);
        printf("%s", a);
        if (a[0] == 'q') {
            break;
        }
    }

    return 0;
}
