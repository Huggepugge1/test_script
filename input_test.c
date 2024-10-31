#include <stdio.h>
#include <stdlib.h>

int main() {
    char *a = malloc(sizeof(char) * 100);
    while (1) {
        scanf("%s", a);
        printf("%s\n", a);
    }
}
