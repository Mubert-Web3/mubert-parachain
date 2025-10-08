import {ApiPromise, WsProvider} from '@polkadot/api';
import {Keyring} from '@polkadot/keyring';

const WS_ENDPOINT_RELAY = process.env.WS_ENDPOINT || 'ws://127.0.0.1:9940';

const WS_ENDPOINT_A = process.env.WS_ENDPOINT || 'ws://127.0.0.1:9944';
const WS_ENDPOINT_B = process.env.WS_ENDPOINT || 'ws://127.0.0.1:9945';

const MNEMONIC = process.env.MNEMONIC || '//Alice';

const HRMP_REQUEST_A = '0x0f001f00030100030c000400000000070010a5d4e8130000000002286bee0006000300c16678419c383c00751200000800000000001000';
const HRMP_ACCEPT_B = '0x0f001f00030100030c000400000000070010a5d4e8130000000002286bee0006000300c16678419c183c0174120000';

const HRMP_REQUEST_B = '0x0f001f00030100030c000400000000070010a5d4e8130000000002286bee0006000300c16678419c383c00741200000800000000001000';
const HRMP_ACCEPT_A = '0x0f001f00030100030c000400000000070010a5d4e8130000000002286bee0006000300c16678419c183c0175120000';

async function callTx(tx, api, signer) {
    await tx.signAndSend(signer, {tip: 1000000});
}

function createCalls(apiA, apiB, reqA, reqB) {
    const callRequestA = apiA.createType('Call', reqA);
    const callAcceptB = apiB.createType('Call', reqB);

    const txRequest = apiA.tx(callRequestA);
    const txAccept = apiB.tx(callAcceptB);

    return {txRequest, txAccept};
}

function sleep(ms) {
    console.log(`waiting: ${ms / 1000} seconds ...`);
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
    console.log(`Connecting to relay ${WS_ENDPOINT_RELAY} ...`);
    const providerRelay = new WsProvider(WS_ENDPOINT_RELAY);
    const apiRelay = await ApiPromise.create({provider: providerRelay});

    const keyring = new Keyring({type: 'sr25519'});
    const signer = keyring.addFromUri(MNEMONIC);

    try {
        console.log("top up balance for Child 4724")
        await apiRelay.tx.balances
            .transferKeepAlive("5Ec4AhPAWhaACZhK62F7vSLYtovExeNF7f81QpEppCdNXVgZ", 100000000000000) // Child 4724
            .signAndSend(signer, {tip: 1000000});
    } catch (e) {
    }
    await sleep(60000);
    try {
        console.log("top up balance for Child 4725")
        await apiRelay.tx.balances
            .transferKeepAlive("5Ec4AhPAiKdLigd7ueir7cyczSaf71JNDNLq3RQPQVfLkRVy", 100000000000000) // Child 4725
            .signAndSend(signer, {tip: 1000000});
    } catch (e) {
    }
    await sleep(60000);

    console.log(`Connecting to A ${WS_ENDPOINT_A} ...`);
    const providerA = new WsProvider(WS_ENDPOINT_A);
    const apiA = await ApiPromise.create({provider: providerA});

    console.log(`Connecting to B ${WS_ENDPOINT_B} ...`);
    const providerB = new WsProvider(WS_ENDPOINT_B);
    const apiB = await ApiPromise.create({provider: providerB});


    const txAB = createCalls(apiA, apiB, HRMP_REQUEST_A, HRMP_ACCEPT_B);

    console.log('Sending signed extrinsic...');

    try {
        console.log("send request for hrmp")
        await callTx(txAB.txRequest, apiA, signer);
    } catch (e) {
    }
    await sleep(60000);
    try {
        console.log("send accept for hrmp")
        await callTx(txAB.txAccept, apiB, signer);
    } catch (e) {
    }
    await sleep(60000);

    const txBA = createCalls(apiB, apiA, HRMP_REQUEST_B, HRMP_ACCEPT_A);

    console.log('Sending signed extrinsic...');

    try {
        console.log("send request for hrmp")
        await callTx(txBA.txRequest, apiB, signer);
    } catch (e) {
    }
    await sleep(60000);
    try {
        console.log("send accept for hrmp")
        await callTx(txBA.txAccept, apiA, signer);
    } catch (e) {
    }
    await sleep(60000);

    let goodA, goodB = false;

    const ch = await apiRelay.query.hrmp.hrmpChannels({"sender": "4725", "recipient": "4724"});
    if (!ch.isEmpty) {
        goodA = true;
        console.log(`hrmp.hrmpChannels(4725, 4724) =`, ch.toHuman?.() ?? ch.toString());
    } else {
        console.log(`hrmp.hrmpChannels(4725 -> 4724) not found`);
    }

    const ch1 = await apiRelay.query.hrmp.hrmpChannels({"sender": "4724", "recipient": "4725"});
    if (!ch1.isEmpty) {
        goodB = true;
        console.log(`hrmp.hrmpChannels(4724, 4725) =`, ch1.toHuman?.() ?? ch1.toString());
    } else {
        console.log(`hrmp.hrmpChannels(4724 -> 4725) not found`);
    }

    if (goodA && goodB) {
        console.log(`All good`);
    } else {
        console.log(`Try one more time`);
    }

    console.log(`exit: ctrl+C`);
}

main().catch(console.error);
