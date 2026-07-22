// Bearer-token authentication shared by the private import APIs. Hashing both
// inputs first gives timingSafeEqual two fixed-size values, so neither a token
// mismatch nor a different token length takes a distinguishable fast path.

const encoder = new TextEncoder();

export async function bearerAuthorized(
  request: Request,
  expected: string | undefined,
): Promise<boolean> {
  if (!expected) return false; // secret unset -> the write path stays closed

  const header = request.headers.get("Authorization");
  const match = header?.match(/^Bearer ([^\s]+)$/);
  if (!match) return false;

  const [providedHash, expectedHash] = await Promise.all([
    crypto.subtle.digest("SHA-256", encoder.encode(match[1])),
    crypto.subtle.digest("SHA-256", encoder.encode(expected)),
  ]);
  return crypto.subtle.timingSafeEqual(providedHash, expectedHash);
}
