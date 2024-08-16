import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { MPL_TOKEN_METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";
import {
    PublicKey,
    Connection,
    TransactionInstruction,
    VersionedTransaction,
    TransactionMessage,
    LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { BN } from "bn.js";
import { ChainTicket } from "../types/chain_ticket";
import * as IDL from "../types/chain_ticket.json";

export const idl: ChainTicket = IDL as ChainTicket;

const EVENT_SEED: string = "event";
const MINT_SEED: string = "mint";
const VAULT_SEED: string = "vault";
const METADATA_SEED: string = "metadata";

export function getEventAddress(authority: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [
            Buffer.from(EVENT_SEED),
            authority.toBuffer()
        ],
        new PublicKey(idl.address)
    );
}

export function getMintAddress(eventAddress: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [
            Buffer.from(MINT_SEED),
            eventAddress.toBuffer()
        ],
        new PublicKey(idl.address)
    );
}

export function getVaultAddress(eventAddress: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [
            Buffer.from(VAULT_SEED),
            eventAddress.toBuffer(),
        ],
        new PublicKey(idl.address),
    );
}

export function getMetadataAddress(mintAddress: PublicKey): [PublicKey, number] {
    const mplPubkey = new PublicKey(MPL_TOKEN_METADATA_PROGRAM_ID);
    return PublicKey.findProgramAddressSync(
        [
            Buffer.from(METADATA_SEED),
            mplPubkey.toBuffer(),
            mintAddress.toBuffer()
        ],
        mplPubkey,
    );
}

export async function burnRefundAll(connection: Connection, wallet: Wallet, refund: boolean): Promise<[string[], string[]]> {
    const chainTicketProgram = new ChainTicketProgram(connection, wallet);
    const eventAddress = getEventAddress(wallet.publicKey)[0];
    const mintAddress = getMintAddress(eventAddress)[0];
    const filter = [
        {
            dataSize: 165, // Token account size
        },
        {
            memcmp: {
                offset: 32, // Offset for the mint address
                bytes: mintAddress.toBase58(),
            },
        },
    ];

    const accounts = await connection.getProgramAccounts(TOKEN_PROGRAM_ID, {
        filters: filter,
    }).then(r => r.map(({ pubkey }) => pubkey));

    let txids: string[] = [];
    let failures: string[] = [];

    await Promise.all(accounts.map(async (buyer) => {
        try {
            let ix: TransactionInstruction;
            if (refund) {
                ix = await chainTicketProgram.getRefundTicketIx(buyer);
            } else {
                ix = await chainTicketProgram.getDelegateBurnIx(buyer);
            }
            const txid = await chainTicketProgram.sendTransaction([ix]);
            txids.push(txid);
        } catch (error) {
            console.error("Error processing buyer:", buyer.toBase58(), error);
            failures.push(buyer.toBase58());
        }
    }));

    return [txids, failures];
}

export async function refundAll(connection: Connection, wallet: Wallet): Promise<[string[], string[]]> {
    return await burnRefundAll(connection, wallet, true);
}

export async function burnAll(connection: Connection, wallet: Wallet): Promise<[string[], string[]]> {
    return await burnRefundAll(connection, wallet, false);
}

export type InitEventFields = {
    eventName: string,
    eventSymbol: string,
    imageUri: string,
    metadataUri: string,
    eventDate: number, // As a unix timestamp
    ticketPrice: number, // In sol, program will convert SOL -> Lamports
    numTickets: number,
    refundPeriod: number, // As a unix timestamp
}

export type AmendEventFields = {
    eventDate: number, // As a unix timestamp
    ticketPrice: number, // In sol, a program will convert SOL -> Lamports
    numTickets: number,
}

export class ChainTicketProgram {
    program: Program<ChainTicket>;

    constructor(connection: Connection, wallet: Wallet) {
        const provider = new AnchorProvider(
            connection,
            wallet,
            {
                preflightCommitment: "confirmed",
                commitment: "confirmed",
            }
        );
        this.program = new Program(idl, provider);
    }

    async sendTransaction(instructions: TransactionInstruction[]): Promise<string> {
        const transaction = await this.prepareTransaction(instructions);
        const txid = await this.program.provider.sendAndConfirm(transaction);

        return txid;
    }

    async prepareTransaction(instructions: TransactionInstruction[]): Promise<VersionedTransaction> {
        const maxRetries = 3;
        const delay = 1000;
        let retries = 0;

        while (retries > maxRetries) {
            try {
                const recentBlockhash = await this.program
                    .provider
                    .connection
                    .getLatestBlockhash()
                    .then(r => r.blockhash);

                const messageV0 = new TransactionMessage({
                    payerKey: this.program.provider.publicKey,
                    recentBlockhash,
                    instructions,
                }).compileToV0Message();

                return new VersionedTransaction(messageV0);
            } catch (error) {
                console.error("Could not get blockhash:", error)
                retries++;
                if (retries < maxRetries) {
                    await new Promise(resolve => setTimeout(resolve, delay));
                } else {
                    throw error;
                }
            }

        }

    }

    getInitEventIx(
        fields: InitEventFields,
    ): Promise<TransactionInstruction> {
        const authority = this.program.provider.publicKey;
        const ticketPrice = fields.ticketPrice * LAMPORTS_PER_SOL;

        return this.program.methods.initEvent({
            eventName: fields.eventName,
            eventSymbol: fields.eventSymbol,
            imageUri: fields.imageUri,
            metadataUri: fields.metadataUri,
            eventDate: new BN(fields.eventDate),
            ticketPrice: new BN(ticketPrice),
            numTickets: fields.numTickets,
            refundPeriod: new BN(fields.refundPeriod),
        })
            .accounts({
                authority
            }).instruction();
    }

    getAmendEventIx(
        fields: AmendEventFields,
    ): Promise<TransactionInstruction> {
        const authority = this.program.provider.publicKey;
        const ticketPrice = fields.ticketPrice * LAMPORTS_PER_SOL;

        return this.program.methods.amendEvent({
            eventDate: new BN(fields.eventDate),
            ticketPrice: new BN(ticketPrice),
            numTickets: fields.numTickets,
        })
            .accounts({
                authority,
            }).instruction();
    }

    getStartSaleIx(): Promise<TransactionInstruction> {
        return this.program.methods.startSale().accounts(
            {
                authority: this.program.provider.publicKey
            }
        ).instruction();
    }

    getBuyTicketIx(event: PublicKey): Promise<TransactionInstruction> {
        return this.program.methods.buyTicket().accounts(
            {
                event,
                buyer: this.program.provider.publicKey,
            }
        ).instruction();
    }

    getRefundTicketIx(buyer: PublicKey): Promise<TransactionInstruction> {
        return this.program.methods.refundTicket().accounts(
            {
                authority: this.program.provider.publicKey,
                buyer,
            }
        ).instruction();
    }

    getBurnTicketIx(event: PublicKey): Promise<TransactionInstruction> {
        return this.program.methods.burnTicket().accounts(
            {
                event,
                ticketHolder: this.program.provider.publicKey,
            }
        ).instruction();
    }

    getDelegateBurnIx(targetWallet: PublicKey): Promise<TransactionInstruction> {
        return this.program.methods.delegateBurn().accounts(
            {
                authority: this.program.provider.publicKey,
                targetWallet,
            }
        ).instruction();
    }

    getWithdrawFundsIx(): Promise<TransactionInstruction> {
        return this.program.methods.withdrawFunds().accounts(
            {
                authority: this.program.provider.publicKey,
            }
        ).instruction();
    }

    getCancelEventIx(): Promise<TransactionInstruction> {
        return this.program.methods.cancelEvent().accounts(
            {
                authority: this.program.provider.publicKey,
            }
        ).instruction();
    }

    getEndEventIx(): Promise<TransactionInstruction> {
        return this.program.methods.endEvent().accounts(
            {
                authority: this.program.provider.publicKey,
            }
        ).instruction();
    }
}
