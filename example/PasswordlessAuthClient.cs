// .NET 7+ example client for interacting with the Passwordless Auth server.
// Build with: dotnet new console -o client && replace Program.cs with this file.
// Then: dotnet add package System.Text.Json
// Usage: adjust email/token values in Main or adapt to CLI.

using System;
using System.Net.Http;
using System.Net.Http.Json;
using System.Text.Json;
using System.Threading.Tasks;

namespace PasswordlessAuthClient
{
    public class AuthClient
    {
        private readonly HttpClient _http;

        public AuthClient(string baseUrl)
        {
            _http = new HttpClient { BaseAddress = new Uri(baseUrl) };
        }

        public async Task RequestMagicLinkAsync(string email)
        {
            var res = await _http.PostAsJsonAsync("/request/magic", new { email });
            Console.WriteLine($"RequestMagicLink status: {res.StatusCode}");
        }

        public async Task<JsonElement?> VerifyMagicLinkAsync(string token)
        {
            var res = await _http.GetAsync($"/verify/magic?token={Uri.EscapeDataString(token)}");
            var text = await res.Content.ReadAsStringAsync();
            if (!res.IsSuccessStatusCode)
            {
                Console.WriteLine("Verify failed: " + text);
                return null;
            }
            var json = JsonSerializer.Deserialize<JsonElement>(text);
            Console.WriteLine("VerifyMagicLink response: " + json);
            return json;
        }

        public async Task<JsonElement?> RefreshTokenAsync(string refreshToken)
        {
            var res = await _http.PostAsJsonAsync("/token/refresh", new { refresh_token = refreshToken });
            var text = await res.Content.ReadAsStringAsync();
            if (!res.IsSuccessStatusCode)
            {
                Console.WriteLine("Refresh failed: " + text);
                return null;
            }
            var json = JsonSerializer.Deserialize<JsonElement>(text);
            Console.WriteLine("RefreshToken response: " + json);
            return json;
        }

        public async Task<JsonElement?> TotpEnrollAsync(string email)
        {
            var res = await _http.PostAsJsonAsync("/totp/enroll", new { email });
            var text = await res.Content.ReadAsStringAsync();
            if (!res.IsSuccessStatusCode)
            {
                Console.WriteLine("TOTP enroll failed: " + text);
                return null;
            }
            var json = JsonSerializer.Deserialize<JsonElement>(text);
            Console.WriteLine("TOTP enroll: " + json);
            return json;
        }

        public async Task<JsonElement?> TotpVerifyAsync(string email, string code)
        {
            var res = await _http.PostAsJsonAsync("/totp/verify", new { email, code });
            var text = await res.Content.ReadAsStringAsync();
            if (!res.IsSuccessStatusCode)
            {
                Console.WriteLine("TOTP verify failed: " + text);
                return null;
            }
            var json = JsonSerializer.Deserialize<JsonElement>(text);
            Console.WriteLine("TOTP verify: " + json);
            return json;
        }

        public static void PrintJwtPayload(string jwt)
        {
            try
            {
                var parts = jwt.Split('.');
                if (parts.Length != 3)
                {
                    Console.WriteLine("Invalid JWT format");
                    return;
                }
                string payload = parts[1];
                payload = payload.Replace('-', '+').Replace('_', '/');
                switch (payload.Length % 4)
                {
                    case 2: payload += "=="; break;
                    case 3: payload += "="; break;
                }
                var bytes = Convert.FromBase64String(payload);
                var json = JsonSerializer.Deserialize<JsonElement>(bytes);
                Console.WriteLine("JWT payload: " + JsonSerializer.Serialize(json, new JsonSerializerOptions { WriteIndented = true }));
            }
            catch (Exception ex)
            {
                Console.WriteLine("Failed to parse JWT: " + ex.Message);
            }
        }
    }

    class Program
    {
        static async Task Main(string[] args)
        {
            var client = new AuthClient("http://localhost:3000");

            string email = "alice@example.com";

            Console.WriteLine($"Requesting magic link for {email}");
            await client.RequestMagicLinkAsync(email);

            Console.WriteLine("Please retrieve the magic link token from email or DB and paste it:");
            var token = Console.ReadLine()?.Trim();
            if (string.IsNullOrWhiteSpace(token))
            {
                Console.WriteLine("No token provided.");
                return;
            }

            var verifyResp = await client.VerifyMagicLinkAsync(token);
            if (verifyResp.HasValue)
            {
                if (verifyResp.Value.TryGetProperty("access_token", out var access))
                {
                    Console.WriteLine("Access token:");
                    Console.WriteLine(access.GetString());
                    AuthClient.PrintJwtPayload(access.GetString());
                }
                if (verifyResp.Value.TryGetProperty("refresh_token", out var refresh))
                {
                    Console.WriteLine("Refresh token:");
                    Console.WriteLine(refresh.GetString());
                }
            }

            // Example TOTP flow
            var totpEnroll = await client.TotpEnrollAsync(email);
            if (totpEnroll.HasValue && totpEnroll.Value.TryGetProperty("secret", out var secretElem))
            {
                string secret = secretElem.GetString();
                Console.WriteLine($"TOTP Secret: {secret}");
                Console.WriteLine("Generate a code from your authenticator and enter it:");
                var code = Console.ReadLine()?.Trim();
                if (!string.IsNullOrEmpty(code))
                {
                    await client.TotpVerifyAsync(email, code);
                }
            }
        }
    }
}
