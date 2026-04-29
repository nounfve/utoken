import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import './sutils.ts/styles/shared.css'
import App from './App.tsx'
import { BrowserRouter, Navigate, Route, Routes } from 'react-router'
import { Account } from './pages/account.tsx'
import { lastbase } from './sutils.ts/current_page.ts'
import { Login } from './pages/login.tsx'
import { SetToken } from './pages/set_token.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter basename={lastbase("/.ui./")}>
      <Routes>
        <Route index element={<></>} />
        <Route path="/.dev" element={<App />} />
        <Route path='/account' element={<Account />} />
        <Route path='/login' element={<Login />} />
        <Route path='/.set_token' element={<SetToken />} />
        <Route path='*' element={<Navigate to="/" />} />
      </Routes>
    </BrowserRouter>
  </StrictMode>,
)
