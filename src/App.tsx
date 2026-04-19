import { Routes, Route, Navigate } from "react-router-dom";
import { Dashboard } from "./pages/Dashboard";
import { CreateWallet } from "./pages/CreateWallet";
import { ImportWallet } from "./pages/ImportWallet";
import { Settings } from "./pages/Settings";
import { WalletDashboard } from "./pages/WalletDashboard";
import { WalletDetails } from "./pages/WalletDetails";
import { SendTransfer } from "./pages/SendTransfer";
import { BackupWallet } from "./pages/BackupWallet";
import { TokenManagement } from "./pages/TokenManagement";
import { WalletConnect } from "./pages/WalletConnect";

function App() {
  return (
    <Routes>
      <Route path="/" element={<Navigate to="/dashboard" replace />} />
      <Route path="/dashboard" element={<Dashboard />} />
      <Route path="/create" element={<CreateWallet />} />
      <Route path="/import" element={<ImportWallet />} />
      <Route path="/settings" element={<Settings />} />
      <Route path="/wallet" element={<WalletDashboard />} />
      <Route path="/wallet/:id" element={<WalletDetails />} />
      <Route path="/send" element={<SendTransfer />} />
      <Route path="/tokens" element={<TokenManagement />} />
      <Route path="/walletconnect" element={<WalletConnect />} />
      <Route path="/backup/:walletId" element={<BackupWallet />} />
    </Routes>
  );
}

export default App;
