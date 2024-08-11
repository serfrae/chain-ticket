import {
    PublicKey,
    TransactionInstruction,
    TransactionMessage,
    VersionedTransaction,
} from "@solana/web3.js";

import { connection } from "./connection";

export async function prepareTransaction(
    instructions: TransactionInstruction[],
    payer: PublicKey,
): Promise<VersionedTransaction> {
    const blockhash = await connection.getLatestBlockhash().then(r => r.blockhash);
    const messageV0 = new TransactionMessage({
        payerKey: payer,
        recentBlockhash: blockhash,
        instructions,
    }).compileToV0Message();
    return new VersionedTransaction(messageV0);
}
