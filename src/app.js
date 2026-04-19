const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// DOM Elements
const connectBtn = document.getElementById('connectBtn');
const walletAddressEl = document.getElementById('walletAddress');
const balanceEl = document.getElementById('balance');
const sendForm = document.getElementById('sendForm');
const sendBtn = document.getElementById('sendBtn');
const tokenType = document.getElementById('tokenType');
const tokenAddressGroup = document.getElementById('tokenAddressGroup');
const tokenAddress = document.getElementById('tokenAddress');
const toAddress = document.getElementById('toAddress');
const amount = document.getElementById('amount');
const gasPrice = document.getElementById('gasPrice');
const gasLimit = document.getElementById('gasLimit');
const nonce = document.getElementById('nonce');
const data = document.getElementById('data');
const estimatedGasEl = document.getElementById('estimatedGas');
const statusMessage = document.getElementById('statusMessage');
const txHash = document.getElementById('txHash');
const loadingOverlay = document.getElementById('loadingOverlay');
const loadingText = document.getElementById('loadingText');

let currentWallet = null;

// Initialize
document.addEventListener('DOMContentLoaded', async () => {
    try {
        await invoke('init_wallet');
        console.log('Wallet initialized');
    } catch (e) {
        console.log('Wallet init skipped or failed:', e);
    }
});

// Connect Wallet
connectBtn.addEventListener('click', async () => {
    try {
        showLoading('Connecting wallet...');
        const address = await invoke('get_wallet_address');
        currentWallet = address;
        walletAddressEl.textContent = address;
        
        await refreshBalance();
        sendBtn.disabled = false;
        
        // Estimate gas on load
        await estimateGas();
        
        hideLoading();
        showStatus('Wallet connected successfully!', 'success');
    } catch (e) {
        hideLoading();
        showStatus(`Connection failed: ${e}`, 'error');
    }
});

// Token Type Change
tokenType.addEventListener('change', () => {
    if (tokenType.value === 'ERC20') {
        tokenAddressGroup.style.display = 'block';
        tokenAddress.required = true;
    } else {
        tokenAddressGroup.style.display = 'none';
        tokenAddress.required = false;
    }
    estimateGas();
});

// Input Changes for Gas Estimation
[toAddress, amount, tokenAddress].forEach(el => {
    el.addEventListener('input', debounce(estimateGas, 500));
});

// Estimate Gas
async function estimateGas() {
    if (!currentWallet || !toAddress.value) return;
    
    try {
        const tokenTypeVal = tokenType.value;
        const toAddr = toAddress.value;
        const amountVal = amount.value || '0';
        const tokenAddr = tokenAddress.value || '';
        
        if (!isValidAddress(toAddr)) return;
        
        const estimate = await invoke('estimate_gas', {
            tokenType: tokenTypeVal,
            toAddress: toAddr,
            amount: amountVal,
            tokenAddress: tokenAddr
        });
        
        estimatedGasEl.textContent = estimate.gasLimit;
        if (!gasLimit.value) {
            gasLimit.placeholder = estimate.gasLimit;
        }
        if (!gasPrice.value) {
            gasPrice.placeholder = estimate.gasPrice;
        }
    } catch (e) {
        console.log('Gas estimate error:', e);
    }
}

// Send Transaction
sendForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    
    if (!currentWallet) {
        showStatus('Please connect your wallet first', 'error');
        return;
    }
    
    try {
        showLoading('Preparing transaction...');
        
        const txParams = {
            tokenType: tokenType.value,
            toAddress: toAddress.value,
            amount: amount.value,
            tokenAddress: tokenAddress.value || null,
            gasPrice: gasPrice.value ? parseFloat(gasPrice.value) : null,
            gasLimit: gasLimit.value ? parseInt(gasLimit.value) : null,
            nonce: nonce.value ? parseInt(nonce.value) : null,
            data: data.value || null
        };
        
        showLoading('Sending transaction...');
        
        const result = await invoke('send_transaction', txParams);
        
        hideLoading();
        showStatus('Transaction submitted successfully!', 'success');
        txHash.innerHTML = `Tx Hash: <a href="#" onclick="openExplorer('${result.txHash}')">${result.txHash}</a>`;
        
        // Reset form
        sendForm.reset();
        
        // Refresh balance
        await refreshBalance();
        
    } catch (e) {
        hideLoading();
        showStatus(`Transaction failed: ${e}`, 'error');
    }
});

// Get Balance
async function refreshBalance() {
    try {
        const balances = await invoke('get_balance');
        balanceEl.textContent = `${balances.eth} ETH`;
        
        if (balances.tokens && balances.tokens.length > 0) {
            const tokensList = balances.tokens.map(t => `${t.balance} ${t.symbol}`).join(', ');
            balanceEl.textContent += ` | Tokens: ${tokensList}`;
        }
    } catch (e) {
        console.log('Balance fetch error:', e);
    }
}

// Validation
toAddress.addEventListener('blur', () => {
    const errorEl = document.getElementById('toAddressError');
    if (toAddress.value && !isValidAddress(toAddress.value)) {
        errorEl.textContent = 'Invalid Ethereum address';
        toAddress.style.borderColor = '#ff6b6b';
    } else {
        errorEl.textContent = '';
        toAddress.style.borderColor = '';
    }
});

// Helper Functions
function isValidAddress(addr) {
    return /^0x[a-fA-F0-9]{40}$/.test(addr);
}

function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

function showLoading(text) {
    loadingText.textContent = text;
    loadingOverlay.classList.add('active');
}

function hideLoading() {
    loadingOverlay.classList.remove('active');
}

function showStatus(message, type) {
    statusMessage.textContent = message;
    statusMessage.className = `status-message ${type}`;
}

function openExplorer(txHash) {
    // Open block explorer for the transaction
    const explorerUrl = `https://etherscan.io/tx/${txHash}`;
    window.open(explorerUrl, '_blank');
}

// Listen for events from backend
listen('tx_status', (event) => {
    const { status, message } = event.payload;
    showStatus(message, status);
});