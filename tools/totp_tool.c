// Simple TOTP generator / verifier (RFC 6238) using SHA1.
// Build with: gcc -o totp_tool tools/totp_tool.c -lcrypto
// Usage:
//   ./totp_tool generate <base32-secret>
//   ./totp_tool verify <base32-secret> <6-digit-code> [window]
//
// Example:
//   ./totp_tool generate JBSWY3DPEHPK3PXP
//   ./totp_tool verify JBSWY3DPEHPK3PXP 123456 1

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <stdint.h>
#include <ctype.h>
#include <openssl/hmac.h>
#include <openssl/evp.h>

static const char *BASE32_ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

int base32_decode(const char *encoded, unsigned char **result, size_t *out_len) {
    size_t len = strlen(encoded);
    size_t buffer = 0;
    int bits_left = 0;
    size_t count = 0;
    unsigned char *bytes = malloc((len * 5 / 8) + 1);
    if (!bytes) return -1;

    for (size_t i = 0; i < len; ++i) {
        char ch = encoded[i];
        if (ch == '=' || ch == ' ') break; // padding or separator

        ch = toupper((unsigned char)ch);
        const char *p = strchr(BASE32_ALPHABET, ch);
        if (!p) continue; // skip invalid

        int val = p - BASE32_ALPHABET;
        buffer <<= 5;
        buffer |= val & 0x1F;
        bits_left += 5;
        if (bits_left >= 8) {
            bytes[count++] = (buffer >> (bits_left - 8)) & 0xFF;
            bits_left -= 8;
        }
    }
    *result = bytes;
    *out_len = count;
    return 0;
}

uint32_t truncate(const unsigned char *hmac_result) {
    int offset = hmac_result[19] & 0x0f;
    uint32_t bin_code = (hmac_result[offset] & 0x7f) << 24 |
                        (hmac_result[offset + 1] & 0xff) << 16 |
                        (hmac_result[offset + 2] & 0xff) << 8 |
                        (hmac_result[offset + 3] & 0xff);
    return bin_code;
}

void compute_totp(const char *base32_secret, int window, char *out_code, size_t digits) {
    unsigned char *secret_bytes = NULL;
    size_t secret_len = 0;
    if (base32_decode(base32_secret, &secret_bytes, &secret_len) != 0) {
        fprintf(stderr, "Failed to decode base32 secret\n");
        exit(1);
    }

    time_t now = time(NULL);
    uint64_t timestep = (uint64_t)(now / 30);

    for (int i = -window; i <= window; ++i) {
        uint64_t t = timestep + i;
        unsigned char msg[8];
        for (int j = 7; j >= 0; --j) {
            msg[j] = t & 0xFF;
            t >>= 8;
        }

        unsigned int len = EVP_MAX_MD_SIZE;
        unsigned char *hmac_result = HMAC(EVP_sha1(), secret_bytes, (int)secret_len,
                                         msg, sizeof(msg), NULL, NULL);
        if (!hmac_result) continue;
        uint32_t code = truncate(hmac_result);
        uint32_t otp = code % (uint32_t)pow(10, digits);
        char candidate[16];
        snprintf(candidate, sizeof(candidate), "%0*u", (int)digits, otp);
        // If exactly zero window, return current code
        if (i == 0) {
            strncpy(out_code, candidate, digits + 1);
            free(secret_bytes);
            return;
        }
        // On verify path, caller will compare
    }
    // default fallback: current
    uint64_t t = timestep;
    unsigned char msg[8];
    for (int j = 7; j >= 0; --j) {
        msg[j] = t & 0xFF;
        t >>= 8;
    }
    unsigned char *hmac_result = HMAC(EVP_sha1(), secret_bytes, (int)secret_len,
                                     msg, sizeof(msg), NULL, NULL);
    uint32_t code = truncate(hmac_result);
    uint32_t otp = code % (uint32_t)pow(10, digits);
    snprintf(out_code, digits + 1, "%0*u", (int)digits, otp);
    free(secret_bytes);
}

int verify_totp(const char *secret, const char *code, int window) {
    char expected[16];
    for (int i = -window; i <= window; ++i) {
        unsigned char *secret_bytes = NULL;
        size_t secret_len = 0;
        if (base32_decode(secret, &secret_bytes, &secret_len) != 0) {
            continue;
        }
        time_t now = time(NULL);
        uint64_t timestep = (uint64_t)(now / 30) + i;
        unsigned char msg[8];
        uint64_t t = timestep;
        for (int j = 7; j >= 0; --j) {
            msg[j] = t & 0xFF;
            t >>= 8;
        }
        unsigned char *hmac_result = HMAC(EVP_sha1(), secret_bytes, (int)secret_len,
                                         msg, sizeof(msg), NULL, NULL);
        uint32_t code_int = truncate(hmac_result);
        uint32_t otp = code_int % 1000000;
        snprintf(expected, sizeof(expected), "%06u", otp);
        free(secret_bytes);
        if (strcmp(expected, code) == 0) return 1;
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "Usage:\n  %s generate <base32-secret>\n  %s verify <base32-secret> <code> [window]\n", argv[0], argv[0]);
        return 1;
    }
    const char *cmd = argv[1];
    const char *secret = argv[2];
    if (strcmp(cmd, "generate") == 0) {
        char code[16] = {0};
        compute_totp(secret, 0, code, 6);
        printf("TOTP: %s\n", code);
    } else if (strcmp(cmd, "verify") == 0) {
        if (argc < 4) {
            fprintf(stderr, "verify needs code\n");
            return 1;
        }
        const char *code = argv[3];
        int window = 1;
        if (argc >= 5) window = atoi(argv[4]);
        int ok = verify_totp(secret, code, window);
        if (ok) {
            printf("VALID\n");
            return 0;
        } else {
            printf("INVALID\n");
            return 2;
        }
    } else {
        fprintf(stderr, "unknown command: %s\n", cmd);
        return 1;
    }
    return 0;
}
