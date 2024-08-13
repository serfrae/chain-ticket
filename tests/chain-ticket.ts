import * as anchor from "@coral-xyz/anchor";
import { assert } from "chai";
import {
    InitEventFields,
    AmendEventFields,
    ChainTicketProgram,
    getEventAddress,
    getMintAddress,
    getVaultAddress,
    idl,
} from "../app/lib/program";
import { TOKEN_PROGRAM_ID, MintLayout, AccountLayout, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";


describe("chain-ticket", () => {
    let chainTicket: ChainTicketProgram;

    before(async () => {
        const provider = anchor.AnchorProvider.env();
        anchor.setProvider(provider);
        chainTicket = new ChainTicketProgram(
            provider.connection,
            provider.wallet as anchor.Wallet
        );
    });

    it("init", async () => {
        const fields: InitEventFields = {
            eventName: "test",
            eventSymbol: "TST",
            imageUri: "https://test.com/",
            metadataUri: "https://testmetadata.com/",
            eventDate: 123123123,
            ticketPrice: 3.3,
            numTickets: 100,
            refundPeriod: 72000,
        };

        const ix = await chainTicket.getInitEventIx(fields);
        await chainTicket.sendTransaction([ix]);

        const eventAddress = getEventAddress(chainTicket.program.provider.publicKey)[0];
        const mintAddress = getMintAddress(eventAddress)[0];
        const vaultAddress = getVaultAddress(eventAddress)[0];

        const accountInfo = await chainTicket.program.account.event
            .fetch(eventAddress);

        const mintAccount = await chainTicket.program.provider.connection
            .getAccountInfo(mintAddress);
        const mintData = MintLayout.decode(mintAccount.data);

        const vaultAccount = await chainTicket.program.provider.connection
            .getAccountInfo(vaultAddress);

        assert.ok(accountInfo.authority.equals(chainTicket.program.provider.publicKey));
        console.log("Event authority: OK");

        assert.ok(accountInfo.mint.equals(mintAddress));
        console.log("Mint address: OK");

        assert.strictEqual(accountInfo.eventDate.toNumber(), fields.eventDate);
        assert.strictEqual(accountInfo.numTickets, fields.numTickets);
        assert.strictEqual(accountInfo.ticketPrice.toNumber() / LAMPORTS_PER_SOL, fields.ticketPrice);
        console.log("Event fields: OK");

        assert.ok(mintAccount !== null, "Mint account not initialised");
        console.log("Mint initialisation: OK");

        assert.ok(mintAccount.owner.equals(TOKEN_PROGRAM_ID));
        console.log("Mint program owner: OK");

        assert.ok(mintData.supply.toString() === "0", "Supply should be zero");
        console.log("Mint supply: OK");

        assert.ok(mintData.mintAuthority.equals(eventAddress));
        console.log("Mint authority: OK");

        assert.ok(mintData.freezeAuthority.equals(eventAddress));
        console.log("Freeze authority: OK");

        assert.ok(vaultAccount !== null, "Vault is not initialised");
        console.log("Vault initialisation: OK");

        assert.ok(vaultAccount.owner.equals(new PublicKey(idl.address)));
        console.log("Vault owner: OK");

    });

    it("amend", async () => {
        const fields: AmendEventFields = {
            eventDate: 9999999,
            ticketPrice: 0.2,
            numTickets: 50,
        };

        const ix = await chainTicket.getAmendEventIx(fields);
        await chainTicket.sendTransaction([ix]);

        const eventAddress = getEventAddress(chainTicket.program.provider.publicKey)[0];

        const accountInfo = await chainTicket.program.account.event
            .fetch(eventAddress);

        assert.strictEqual(accountInfo.eventDate.toNumber(), fields.eventDate);
        assert.strictEqual(accountInfo.ticketPrice.toNumber() / LAMPORTS_PER_SOL, fields.ticketPrice);
        assert.strictEqual(accountInfo.numTickets, fields.numTickets);
        console.log("Event fields: OK");
    });

    it("start", async () => {
        const ix = await chainTicket.getStartSaleIx();
        await chainTicket.sendTransaction([ix]);

        const eventAddress = getEventAddress(chainTicket.program.provider.publicKey)[0];

        const accountInfo = await chainTicket.program.account.event
            .fetch(eventAddress);
        assert.strictEqual(accountInfo.allowPurchase, true);
    });

    it("buy", async () => {
        const eventAddress = getEventAddress(chainTicket.program.provider.publicKey)[0];
        const ix = await chainTicket.getBuyTicketIx(eventAddress);
        await chainTicket.sendTransaction([ix]);

        const mintAddress = getMintAddress(eventAddress)[0];
        const ata = getAssociatedTokenAddressSync(mintAddress, chainTicket.program.provider.publicKey);

        const ataInfo = await chainTicket.program.provider.connection.getAccountInfo(ata);
        const mintInfo = await chainTicket.program.provider.connection.getAccountInfo(mintAddress);

        const ataData = AccountLayout.decode(ataInfo.data);
        const mintData = MintLayout.decode(mintInfo.data);
        assert.strictEqual(ataData.amount.toString(), "1");
        console.log("ATA amount: OK");
        assert.strictEqual(mintData.supply.toString(), "1");
        console.log("Mint supply: OK");

    });

    it("refund", async () => {
        const ix = await chainTicket.getRefundTicketIx(chainTicket.program.provider.publicKey);
        await chainTicket.sendTransaction([ix]);

        const eventAddress = getEventAddress(chainTicket.program.provider.publicKey)[0];
        const mintAddress = getMintAddress(eventAddress)[0];
        const ata = getAssociatedTokenAddressSync(mintAddress, chainTicket.program.provider.publicKey);

        const ataInfo = await chainTicket.program.provider.connection.getAccountInfo(ata);
        const mintInfo = await chainTicket.program.provider.connection.getAccountInfo(mintAddress);

        const ataData = AccountLayout.decode(ataInfo.data);
        const mintData = MintLayout.decode(mintInfo.data);

        assert.strictEqual(ataData.amount.toString(), "0");
        assert.strictEqual(mintData.supply.toString(), "0");
        console.log("ATA and Mint Supply: OK");
    });

    it("burn", async () => {
        const buy = await chainTicket.getBuyTicketIx(
            getEventAddress(chainTicket.program.provider.publicKey)[0]
        );
        await chainTicket.sendTransaction([buy]);
        const ix = await chainTicket.getBurnTicketIx(
            getEventAddress(chainTicket.program.provider.publicKey)[0]
        );
        await chainTicket.sendTransaction([ix]);

    });

    it("delegate burn", async () => {
        const buy = await chainTicket.getBuyTicketIx(
            getEventAddress(chainTicket.program.provider.publicKey)[0]
        );
        await chainTicket.sendTransaction([buy]);

        const ix = await chainTicket.getDelegateBurnIx(chainTicket.program.provider.publicKey);
        await chainTicket.sendTransaction([ix]);

        const eventAddress = getEventAddress(chainTicket.program.provider.publicKey)[0];
        const mintAddress =  getMintAddress(eventAddress)[0];
        const ata = getAssociatedTokenAddressSync(mintAddress, chainTicket.program.provider.publicKey);

        const ataInfo = await chainTicket.program.provider.connection.getAccountInfo(ata);
        const mintInfo = await chainTicket.program.provider.connection.getAccountInfo(mintAddress);

        const ataData = AccountLayout.decode(ataInfo.data);
        const mintData = MintLayout.decode(mintInfo.data);

        assert.strictEqual(ataData.amount.toString(), "0");
        assert.strictEqual(mintData.supply.toString(), "0");

    });

    //it("withdraw", async () => {
    //    const ix = await chainTicket.getWithdrawFundsIx();
    //    const txid = await chainTicket.sendTransaction([ix]);
    //    console.log("TXID:", txid);
    //});

    //it("cancel", async () => {
    //    const ix = await chainTicket.getCancelEventIx();
    //    const txid = await chainTicket.sendTransaction([ix]);
    //    console.log("TXID:", txid);
    //});

    it("end", async () => {
        const ix = await chainTicket.getEndEventIx();
        await chainTicket.sendTransaction([ix]);
    });
});
