import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { useWalletStore } from "./stores/walletStore";
import WalletDashboard from "./pages/WalletDashboard";
import CreateWallet from "./pages/CreateWallet";
import ImportWallet from "./pages/ImportWallet";
import WalletDetails from "./pages/WalletDetails";
import "./index.css";

function App() {
  const { activeWallet } = useWalletStore();

  return (
    <BrowserRouter>
      <div className="min-h-screen bg-vault-bg text-vault-text">
        {/* Header */}
        <header className="bg-vault-card border-b border-vault-border px-6 py-4">
          <div className="max-w-6xl mx-auto flex items-center justify-between">
            <h1 className="text-xl font-bold bg-vault-gradient bg-clip-text text-transparent">
              TokenVault
            </h1>
            {activeWallet && (
              <div className="flex items-center gap-2 text-sm text-vault-text-secondary">
                <span>Active:</span>
                <span className="font-mono text-vault-text">
                  {activeWallet.address.slice(0, 6)}...{activeWallet.address.slice(-4)}
                </span>
              </div>
            )}
          </div>
        </header>

        {/* Main Content */}
        <main className="max-w-6xl mx-auto p-6">
          <Routes>
            <Route path="/" element={<WalletDashboard />} />
            <Route path="/create" element={<CreateWallet />} />
            <Route path="/import" element={<ImportWallet />} />
            <Route path="/wallet/:id" element={<WalletDetails />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>

        {/* Navigation */}
        <nav className="fixed bottom-0 left-0 right-0 bg-vault-card border-t border-vault-border">
          <div className="max-w-6xl mx-auto flex justify-around py-3">
            <NavLink to="/" icon="📦" label="Wallets" />
            <NavLink to="/create" icon="➕" label="Create" />
            <NavLink to="/import" icon="📥" label="Import" />
          </div>
        </nav>
      </div>
    </BrowserRouter>
  );
}

function NavLink({ to, icon, label }: { to: string; icon: string; label: string }) {
  return (
    <a
      href={to}
      className="flex flex-col items-center gap-1 text-vault-text-secondary hover:text-vault-text transition-colors"
    >
      <span className="text-xl">{icon}</span>
      <span className="text-xs">{label}</span>
    </a>
  );
}

export default App;
