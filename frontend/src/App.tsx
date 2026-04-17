import { Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore, useUIStore } from './stores';
import { AppLayout } from './components/Layout/AppLayout';
import { LoginPage } from './components/Layout/LoginPage';
import { Library } from './components/Layout/Library';
import { ReaderView } from './components/Reader/ReaderView';
import { GraphFullView } from './components/Graph/GraphFullView';
import { SettingsView } from './components/Settings/SettingsView';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore(s => s.isAuthenticated);
  if (!isAuthenticated) return <Navigate to="/login" replace />;
  return <>{children}</>;
}

export default function App() {
  const theme = useUIStore(s => s.theme);

  return (
    <div data-theme={theme} className="min-h-screen" style={{ background: 'var(--bg-primary)', color: 'var(--text-primary)' }}>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/" element={
          <ProtectedRoute>
            <AppLayout>
              <Library />
            </AppLayout>
          </ProtectedRoute>
        } />
        <Route path="/read/:bookId" element={
          <ProtectedRoute>
            <ReaderView />
          </ProtectedRoute>
        } />
        <Route path="/graph" element={
          <ProtectedRoute>
            <AppLayout>
              <GraphFullView />
            </AppLayout>
          </ProtectedRoute>
        } />
        <Route path="/settings" element={
          <ProtectedRoute>
            <AppLayout>
              <SettingsView />
            </AppLayout>
          </ProtectedRoute>
        } />
      </Routes>
    </div>
  );
}
