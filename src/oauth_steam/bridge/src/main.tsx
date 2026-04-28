import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { BrowserRouter, Navigate, Route, Routes } from 'react-router'
import { SteamAccount } from './pages/account.tsx'
import { SteamCallback } from "./pages/callback.tsx"
import { lastbase } from './utils/current_page.ts'


createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter basename={lastbase('/bridge/')}>
      <Routes>
        <Route path='/' element={<App />} />
        <Route path='/account' element={<SteamAccount />} />
        <Route path='/callback' element={<SteamCallback />} />
        <Route path='*' element={<Navigate to='/' />} />
      </Routes>
    </BrowserRouter>
  </StrictMode>,
)
