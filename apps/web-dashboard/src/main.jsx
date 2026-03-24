import "@fontsource/inter/index.css";
import "@fontsource/jetbrains-mono/index.css";
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import './i18n/config' // 核心：导入国际化配置
import App from './App.jsx'

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
