// This file is included in albedo.rs

export async function publicKey(token) {
    return albedo.publicKey({
        token: token
    });
}

export async function signMessage(message) {
    return albedo_sign_message(message);
}

export async function signMessagePubKey(message, pubKey)
{
    return albedo_sign_message(message, pubKey);
}

export async function albedo_view_public_key(token = "", callback = "", require_existing = false) {
    return albedo.publicKey({
        token: token,
        callback: callback,
        require_existing: require_existing
    });
}

export async function albedo_sign_message(message, pubkey = "", callback = "") {
    return albedo.signMessage({
        message: message,
        pubkey: pubkey,
        callback: callback
    });
}

export async function albedo_sign_transaction(xdr, pubkey = "", network = "testnet", callback = "", submit = false)
{
    return albedo.tx({
        xdr: xdr,
        pubkey: pubkey,
        network: network,
        callback: callback,
        submit: submit
    });
}

export async function albedo_make_payment(amount, destination, asset_code = "", asset_issuer = "", memo = "", memo_type = "",
                                            pubkey = "", network = "testnet", callback = "", submit = false)
{
    return albedo.pay({
        amount: amount,
        destination: destination,
        asset_code: asset_code,
        asset_issuer: asset_issuer,
        memo: memo,
        memo_type: memo_type,
        pubkey: pubkey,
        network: network,
        callback: callback,
        submit: submit
    })
}

export async function albedo_establish_trustline(asset_code = "", asset_issuer = "", limit = "", memo = "", memo_type = "",
                                            pubkey = "", network = "testnet", callback = "", submit = false)
{
    return albedo.trust({
        asset_code: asset_code,
        asset_issuer: asset_issuer,
        limit: limit,
        memo: memo,
        memo_type: memo_type,
        pubkey: pubkey,
        network: network,
        callback: callback,
        submit: submit
    })
}