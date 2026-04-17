import { useNavigate, useLocation } from 'react-router-dom';
import { BookOpen, Network, Settings, LogOut, Menu, X } from 'lucide-react';
import { useAuthStore, useUIStore } from '../../stores';
import { clsx } from 'clsx';

interface Props {
  children: React.ReactNode;
}

export function AppLayout({ children }: Props) {
  const navigate = useNavigate();
  const location = useLocation();
  const { logout } = useAuthStore();
  const { sidebarOpen, toggleSidebar } = useUIStore();

  const navItems = [
    { path: '/', icon: BookOpen, label: 'Library' },
    { path: '/graph', icon: Network, label: 'Knowledge Graph' },
    { path: '/settings', icon: Settings, label: 'Settings' },
  ];

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Sidebar */}
      <aside
        className={clsx(
          'flex flex-col border-r transition-all duration-300 relative noise-overlay',
          sidebarOpen ? 'w-56' : 'w-16'
        )}
        style={{ borderColor: 'var(--border)', background: 'var(--bg-secondary)' }}
      >
        <div className="relative z-10 flex flex-col h-full">
          {/* Logo */}
          <div className="flex items-center gap-3 px-4 h-16 border-b" style={{ borderColor: 'var(--border)' }}>
            <button onClick={toggleSidebar} className="focus-ring rounded p-1" style={{ color: 'var(--text-muted)' }}>
              {sidebarOpen ? <X size={18} /> : <Menu size={18} />}
            </button>
            {sidebarOpen && (
              <h1 className="font-display text-xl font-bold tracking-tight" style={{ color: 'var(--accent)' }}>
                Nodum
              </h1>
            )}
          </div>

          {/* Nav */}
          <nav className="flex-1 py-4 px-2 space-y-1">
            {navItems.map(item => {
              const active = location.pathname === item.path;
              return (
                <button
                  key={item.path}
                  onClick={() => navigate(item.path)}
                  className={clsx(
                    'flex items-center gap-3 w-full px-3 py-2.5 rounded-lg text-sm font-medium transition-smooth focus-ring',
                    active ? 'text-white' : ''
                  )}
                  style={{
                    background: active ? 'var(--accent)' : 'transparent',
                    color: active ? '#fff' : 'var(--text-secondary)',
                  }}
                  title={item.label}
                >
                  <item.icon size={18} />
                  {sidebarOpen && <span>{item.label}</span>}
                </button>
              );
            })}
          </nav>

          {/* Logout */}
          <div className="px-2 pb-4">
            <button
              onClick={() => { logout(); navigate('/login'); }}
              className="flex items-center gap-3 w-full px-3 py-2.5 rounded-lg text-sm transition-smooth focus-ring"
              style={{ color: 'var(--text-muted)' }}
              title="Log out"
            >
              <LogOut size={18} />
              {sidebarOpen && <span>Log out</span>}
            </button>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-auto" style={{ background: 'var(--bg-primary)' }}>
        {children}
      </main>
    </div>
  );
}
