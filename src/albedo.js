// This file is included in albedo.rs

export async function publicKey(token) {
    return albedo.publicKey({
        token: token
    });
}
