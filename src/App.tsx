import { Routes, Route, Navigate } from "react-router-dom";
import { Dashboard } from "./pages/Dashboard";
import { CreateWallet } from "./pages/CreateWallet";
import { ImportWallet } from "./pages/ImportWallet";
import { Settings } from "./pages/Settings";
import { WalletDashboard } from "./pages/WalletDashboard";
import { WalletDetails } from "./pages/WalletDetails";

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
    </Routes>
  );
}

export default App;
