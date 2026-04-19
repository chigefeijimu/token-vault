import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { WalletDashboard } from './pages/WalletDashboard'
import { CreateWallet } from './pages/CreateWallet'
import { ImportWallet } from './pages/ImportWallet'
import { WalletDetails } from './pages/WalletDetails'

function App() {
  return (
    <BrowserRouter>
      <div className="min-h-screen bg-gray-900 text-white">
        <Routes>
          <Route path="/" element={<WalletDashboard />} />
          <Route path="/create" element={<CreateWallet />} />
          <Route path="/import" element={<ImportWallet />} />
          <Route path="/wallet/:id" element={<WalletDetails />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </div>
    </BrowserRouter>
  )
}

export default App
