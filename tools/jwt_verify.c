// Simple JWT HS256 verifier. Prints payload if signature valid.
// Build with: gcc -o jwt_verify tools/jwt_verify.c -lcrypto
// Usage: ./jwt_verify <jwt> <secret>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/hmac.h>
#include <openssl/evp.h>

// base64url decode
unsigned char *b64url_decode(const char *input, size_t *out_len) {
    size_t len = strlen(input);
    // make a modifiable copy and replace URL-safe
    char *tmp = malloc(len + 5);
    if (!tmp) return NULL;
    strcpy(tmp, input);
    for (size_t i = 0; i < len; ++i) {
        if (tmp[i] == '-') tmp[i] = '+';
        else if (tmp[i] == '_') tmp[i] = '/';
    }
    // pad with '='
    size_t pad = (4 - (len % 4)) % 4;
    for (size_t i = 0; i < pad; ++i) strcat(tmp, "=");
    BIO *b64 = BIO_new(BIO_f_base64());
    BIO_set_flags(b64, BIO_FLAGS_BASE64_NO_NL);
    BIO *bio = BIO_new_mem_buf(tmp, -1);
    bio = BIO_push(b64, bio);
    unsigned char *buffer = malloc(len);
    int decoded_len = BIO_read(bio, buffer, len);
    if (decoded_len <= 0) {
        free(buffer);
        buffer = NULL;
        decoded_len = 0;
    }
    BIO_free_all(bio);
    free(tmp);
    *out_len = decoded_len;
    return buffer;
}

int constant_time_cmp(const unsigned char *a, const unsigned char *b, size_t len) {
    unsigned char diff = 0;
    for (size_t i = 0; i < len; ++i) {
        diff |= a[i] ^ b[i];
    }
    return diff == 0;
}

int main(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <jwt> <secret>\n", argv[0]);
        return 1;
    }
    char *jwt = argv[1];
    char *secret = argv[2];

    // split parts
    char *dot1 = strchr(jwt, '.');
    if (!dot1) { fprintf(stderr, "invalid jwt\n"); return 1; }
    char *dot2 = strchr(dot1 + 1, '.');
    if (!dot2) { fprintf(stderr, "invalid jwt\n"); return 1; }

    size_t header_len = dot1 - jwt;
    size_t payload_len = dot2 - dot1 -1;
    size_t sig_len = strlen(dot2 +1);

    char *header_b64 = strndup(jwt, header_len);
    char *payload_b64 = strndup(dot1 +1, payload_len);
    char *sig_b64 = strdup(dot2 +1);

    // reconstruct signing input
    size_t signing_input_len = header_len + 1 + payload_len;
    char *signing_input = malloc(signing_input_len +1);
    snprintf(signing_input, signing_input_len +1, "%s.%s", header_b64, payload_b64);

    // decode signature
    size_t sig_dec_len;
    unsigned char *sig_dec = b64url_decode(sig_b64, &sig_dec_len);

    // compute HMAC SHA256
    unsigned int result_len;
    unsigned char *hmac = HMAC(EVP_sha256(), secret, strlen(secret),
                               (unsigned char *)signing_input, strlen(signing_input),
                               NULL, &result_len);
    if (!hmac) {
        fprintf(stderr, "HMAC failed\n");
        return 1;
    }

    int valid = 0;
    if (sig_dec && result_len == sig_dec_len) {
        if (constant_time_cmp(hmac, sig_dec, result_len)) {
            valid = 1;
        }
    }

    if (!valid) {
        printf("Signature: INVALID\n");
        return 2;
    }
    printf("Signature: VALID\n");

    // decode payload
    size_t pl_dec_len;
    unsigned char *pl_dec = b64url_decode(payload_b64, &pl_dec_len);
    if (pl_dec) {
        printf("Payload: %.*s\n", (int)pl_dec_len, pl_dec);
        free(pl_dec);
    }

    free(header_b64);
    free(payload_b64);
    free(sig_b64);
    free(signing_input);
    if (sig_dec) free(sig_dec);
    return 0;
}
