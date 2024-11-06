#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    char *a = malloc(sizeof(char) * 100);
    while (1) {
        scanf("%s", a);
        printf("%s\n", a);
        if (strcmp(a, "q") == 0) {
            break;
        }
    }
}
