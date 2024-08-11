import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import { MPL_TOKEN_METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";
import { PublicKey, Connection, TransactionInstruction } from "@solana/web3.js";
import { BN } from "bn.js";
import { IDL, ChainTicket } from "../types/chain_ticket";

const EVENT_SEED: string = "event";
const MINT_SEED: string = "mint";
const METADATA_SEED: string = "metadata";

export function getEventAddress(authority: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [Buffer.from(EVENT_SEED), authority.toBuffer()],
        new PublicKey(IDL.address)
    );
}

export function getMintAddress(eventAddress: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [
            Buffer.from(MINT_SEED),
            eventAddress.toBuffer()
        ],
        new PublicKey(IDL.address)
    );
}

export function getMetadataAddress(mintAddress: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [
            Buffer.from(METADATA_SEED),
            MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
            mintAddress.toBuffer()
        ],
        MPL_TOKEN_METADATA_PROGRAM_ID
    );
}

type InitEventFields = {
    eventName: string,
    eventSymbol: string,
    imageUri: string,
    metadataUri: string,
    eventDate: number,
    ticketPrice: number,
    numTickets: number,
    refundPeriod: number,
}

type AmendEventFields = {
    eventDate: number,
    ticketPrice: number,
    numTickets: number,
}

export default class ChainTicketProgram {
    private program: Program<ChainTicket>;

    constructor(connection: Connection, wallet: Wallet) {
        const provider = new AnchorProvider(
            connection,
            wallet,
            {
                preflightCommitment: "recent",
                commitment: "confirmed",
            }
        );
        this.program = new Program(IDL, provider);
    }

    getInitEventIx(
        fields: InitEventFields,
    ): Promise<TransactionInstruction> {
        const authority = this.program.provider.publicKey;

        return this.program.methods.initEvent({
            eventName: fields.eventName,
            eventSymbol: fields.eventSymbol,
            imageUri: fields.imageUri,
            metadataUri: fields.metadataUri,
            eventDate: new BN(fields.eventDate),
            ticketPrice: new BN(fields.ticketPrice),
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

        return this.program.methods.amendEvent({
            eventDate: new BN(fields.eventDate),
            ticketPrice: new BN(fields.ticketPrice),
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

    getBuyTicketIx(): Promise<TransactionInstruction> {
        return this.program.methods.buyTicket().accounts(
            {
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

    getBurnTicketIx(): Promise<TransactionInstruction> {
        return this.program.methods.burnTicket().accounts(
            {
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
