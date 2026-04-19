# TokenVault - 加密货币钱包

## 1. Concept & Vision

TokenVault 是一款面向DeFi玩家的桌面端加密货币钱包，专注于安全、便捷的资产管理。设计理念是"专业但不复杂"——界面简洁但功能强大，为DeFi玩家提供一站式的链上资产管理体验。

**核心价值**：
- 安全：开源、端侧存储、用户完全控制私钥
- 高效：支持多链切换、代币批量管理
- 沉浸：DApp浏览器无缝连接DeFi世界

## 2. Design Language

### Aesthetic Direction
暗色系专业金融风格，参考 Binance App + MetaMask 的设计融合。深邃的暗紫色调体现加密原生感，辅以渐变色高亮展示活跃状态。

### Color Palette
```
Primary Background: #0D0D1A (深邃背景)
Secondary Background: #1A1A2E (卡片背景)
Accent Gradient: linear-gradient(135deg, #667EEA 0%, #764BA2 100%)
Success: #10B981
Warning: #F59E0B  
Error: #EF4444
Text Primary: #FFFFFF
Text Secondary: #A0AEC0
Border: #2D2D44
```

### Typography
- 标题：Inter, 700
- 正文：Inter, 400/500
- 地址/哈希：JetBrains Mono (等宽)

### Motion Philosophy
- 页面切换：fade + slide，200ms ease-out
- 按钮交互：scale 0.98 on press，150ms
- 数据加载：skeleton shimmer动画
- Toast通知：从右侧滑入，300ms

## 3. Features

### 3.1 钱包管理
- **创建钱包**：生成随机助记词（12/24词），可自定义密码
- **导入钱包**：支持助记词、私钥、Keystore JSON导入
- **导出钱包**：导出Keystore，密码验证后下载
- **钱包列表**：多钱包管理，显示名称/地址/余额
- **切换钱包**：一键切换当前活跃钱包
- **备份提醒**：首次使用提醒用户备份助记词

### 3.2 多链支持（EVM）
- **支持链**：
  - Ethereum Mainnet
  - Binance Smart Chain (BNB Chain)
  - Polygon
  - Arbitrum
  - Optimism
  - Avalanche C-Chain
- **链切换**：顶部网络选择器，支持自定义RPC
- **余额展示**：原生代币余额 + 总估值（USD）

### 3.3 代币管理
- **代币列表**：显示当前链所有代币余额
- **搜索代币**：按合约地址或符号搜索
- **添加自定义代币**：输入合约地址自动获取代币信息
- **隐藏代币**：隐藏零余额代币
- **代币价格**：显示USD估值

### 3.4 转账功能
- **发送资产**：选择代币，输入地址/金额
- **地址簿**：保存常用收款地址
- **Gas设置**：手动调节Gas Price/Gas Limit
- **交易历史**：显示所有交易记录及状态
- **交易确认**：显示详情+密码确认

### 3.5 DApp浏览器
- **内置浏览器**：Web3注入（window.ethereum）
- **DApp列表**：常用DeFi DApp快捷访问
- **连接管理**：查看已连接DApp，断开连接
- **签名请求**：交易签名确认弹窗
- **URL导航**：支持输入URL或搜索

## 4. Technical Stack

### Frontend
- **Framework**: React 18 + TypeScript
- **State**: Zustand
- **Styling**: TailwindCSS
- **Web3**: ethers.js v6 + @web3-react
- **UI Components**: 自建 + HeadlessUI

### Backend (Tauri/Rust)
- **Framework**: Tauri 2.x
- **Cryptography**: rustic, ethers-core
- **Key Derivation**: rcgen, eth-keystore
- **Storage**: serde + tokio
- **HTTP**: reqwest (RPC调用)

### Security
- 私钥永不离开Rust层
- 密码使用 Argon2 加密存储
- 所有RPC请求经过安全筛选

## 5. Development Phases

### Phase 1: 项目初始化
- [ ] Tauri + React 项目搭建
- [ ] 基础UI框架
- [ ] 主题和样式系统

### Phase 2: 钱包核心
- [ ] 钱包创建/导入/导出
- [ ] 助记词生成和验证
- [ ] 密码学基础

### Phase 3: 链交互
- [ ] EVM链连接
- [ ] 余额查询
- [ ] 代币列表

### Phase 4: 转账功能
- [ ] 发送交易
- [ ] Gas估算
- [ ] 交易历史

### Phase 5: DApp浏览器
- [ ] Web3 Provider
- [ ] 签名请求
- [ ] DApp连接管理
