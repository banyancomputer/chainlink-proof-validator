const Web3 = require("web3");

async function scan(message) {
    process.stdout.write(message);
    return await new Promise(function(resolve, reject) {
        process.stdin.resume();
        process.stdin.once("data", function(data) {
            process.stdin.pause();
            resolve(data.toString().trim());
        });
    });
}

async function getGasPrice(web3) {
    while (true) {
        const nodeGasPrice = await web3.eth.getGasPrice();
        const userGasPrice = await scan(`Enter gas-price or leave empty to use ${nodeGasPrice}: `);
        if (/^\d+$/.test(userGasPrice))
            return userGasPrice;
        if (userGasPrice == "")
            return nodeGasPrice;
        console.log("Illegal gas-price");
    }
}

async function getTransactionReceipt(web3) {
    while (true) {
        const hash = await scan("Enter transaction-hash or leave empty to retry: ");
        if (/^0x([0-9A-Fa-f]{64})$/.test(hash)) {
            const receipt = await web3.eth.getTransactionReceipt(hash);
            if (receipt)
                return receipt;
            console.log("Invalid transaction-hash");
        }
        else if (hash) {
            console.log("Illegal transaction-hash");
        }
        else {
            return null;
        }
    }
}

async function send(web3, account, transaction) {
    while (true) {
        try {
            const options = {
                to      : transaction._parent._address,
                data    : transaction.encodeABI(),
                gas     : await transaction.estimateGas({from: account.address}),
                gasPrice: await getGasPrice(web3),
            };
            const signed  = await web3.eth.accounts.signTransaction(options, account.privateKey);
            const receipt = await web3.eth.sendSignedTransaction(signed.rawTransaction);
            return receipt;
        }
        catch (error) {
            console.log(error.message);
            const receipt = await getTransactionReceipt(web3);
            if (receipt)
                return receipt;
        }
    }
}

async function run() {

    const NODE_ADDRESS = process.env.API_KEY;
    const PRIVATE_KEY  = process.env.PRIVATE_KEY;
    const web3        = new Web3(NODE_ADDRESS);
    console.log(web3.eth.accounts);
    const account     = web3.eth.accounts.privateKeyToAccount(PRIVATE_KEY);
    const contract_abi = [
        {
            "anonymous": false,
            "inputs": [
                {
                    "indexed": true,
                    "internalType": "uint256",
                    "name": "offerId",
                    "type": "uint256"
                },
                {
                    "indexed": true,
                    "internalType": "uint256",
                    "name": "blockNumber",
                    "type": "uint256"
                },
                {
                    "indexed": false,
                    "internalType": "bytes",
                    "name": "proof",
                    "type": "bytes"
                }
            ],
            "name": "ProofAdded",
            "type": "event"
        },
        {
            "inputs": [
                {
                    "internalType": "bytes",
                    "name": "_proof",
                    "type": "bytes"
                }
            ],
            "name": "save_proof",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]

    const contract_address = '0xd9145CCE52D386f254917e481eB44e9943F39138';
    const contract    = new web3.eth.Contract(contract_abi, contract_address);
    const transaction = contract.methods.signAgreement();
    const receipt     = await send(web3, account, transaction);
    console.log(JSON.stringify(receipt, null, 4));
    if (web3.currentProvider.constructor.name == "WebsocketProvider")
        web3.currentProvider.connection.close();
}

run();