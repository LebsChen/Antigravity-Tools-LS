import React from 'react';
import { createPortal } from 'react-dom';
import { useTranslation } from 'react-i18next';
import { CheckCircle2, AlertCircle, Info, X, AlertTriangle, Bell, Trash2, CheckCheck } from 'lucide-react';
import useAppStore from '../store/useAppStore';

const NotificationCenter = ({ isOpen, onClose }) => {
  const { t } = useTranslation();
  const { notifications, markAsRead, markAllAsRead, clearNotifications } = useAppStore();

  if (!isOpen) return null;

  const getIcon = (type) => {
    switch (type) {
      case 'success': return <CheckCircle2 className="w-4 h-4 text-emerald-500" />;
      case 'error': return <AlertCircle className="w-4 h-4 text-rose-500" />;
      case 'warning': return <AlertTriangle className="w-4 h-4 text-amber-500" />;
      default: return <Info className="w-4 h-4 text-blue-500" />;
    }
  };

  const getTimeAgo = (time) => {
    const seconds = Math.floor((new Date() - new Date(time)) / 1000);
    if (seconds < 60) return t('common.justNow') || '刚刚';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}${t('common.minsAgo') || '分钟前'}`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}${t('common.hoursAgo') || '小时前'}`;
    return new Date(time).toLocaleDateString();
  };

  return createPortal(
    <>
      <div 
        className="fixed inset-0 z-[1000] bg-transparent" 
        onClick={onClose}
      />
      <div className="fixed top-20 right-10 w-80 max-h-[500px] z-[1001] glass-card shadow-2xl rounded-2xl overflow-hidden flex flex-col animate-fade-in border-foreground/10 outline-none">
        <div className="px-5 py-4 border-b border-foreground/5 flex justify-between items-center bg-foreground/[0.02]">
          <h3 className="text-sm font-bold tracking-tight opacity-50 uppercase flex items-center gap-2">
            <Bell className="w-3.5 h-3.5" />
            {t('header.notifications') || '通知中心'}
          </h3>
          <div className="flex gap-1">
            <button 
              onClick={markAllAsRead}
              className="w-7 h-7 flex items-center justify-center rounded-lg hover:bg-foreground/5 transition-colors opacity-30 hover:opacity-100"
              title={t('common.markAllRead') || '全部标记为已读'}
            >
              <CheckCheck className="w-3.5 h-3.5" />
            </button>
            <button 
              onClick={clearNotifications}
              className="w-7 h-7 flex items-center justify-center rounded-lg hover:bg-foreground/5 transition-colors opacity-30 hover:opacity-100"
              title={t('common.clearAll') || '清除全部'}
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        <div className="overflow-y-auto flex-1 custom-scrollbar">
          {notifications.length === 0 ? (
            <div className="py-12 flex flex-col items-center justify-center opacity-20 gap-3">
              <Bell className="w-8 h-8 stroke-[1]" />
              <p className="text-[11px] font-bold uppercase tracking-widest">{t('common.noNotifications') || '暂无通知'}</p>
            </div>
          ) : (
            notifications.map((n) => (
              <div 
                key={n.id} 
                onClick={() => markAsRead(n.id)}
                className={`px-5 py-4 border-b border-foreground/[0.03] last:border-0 cursor-pointer transition-all duration-300 relative group ${!n.read ? 'bg-foreground/[0.02]' : 'opacity-60 grayscale-[0.5]'}`}
              >
                {!n.read && (
                  <div className="absolute left-2.5 top-1/2 -translate-y-1/2 w-1 h-1 bg-orange-500 rounded-full shadow-[0_0_8px_rgba(249,115,22,0.8)]" />
                )}
                <div className="flex gap-3">
                   <div className="shrink-0 mt-0.5">{getIcon(n.type)}</div>
                   <div className="flex-1 space-y-1">
                       <div className="flex justify-between items-start gap-2">
                        <h4 className={`text-[13px] leading-tight font-bold ${!n.read ? 'text-foreground' : 'text-foreground/60'}`}>
                          {n.title || (n.type === 'error' ? t('common.error') : t('header.notifications'))}
                        </h4>
                        <span className="text-[10px] whitespace-nowrap opacity-30 font-mono tracking-tighter">{getTimeAgo(n.time)}</span>
                      </div>
                      <p className={`text-[12px] leading-relaxed text-foreground/50 ${n.title ? 'line-clamp-2' : 'line-clamp-3'}`}>{n.content}</p>
                   </div>
                </div>
              </div>
            ))
          )}
        </div>

        <div className="px-5 py-3 bg-foreground/[0.03] flex justify-center border-t border-foreground/5">
          <button 
            onClick={onClose}
            className="text-[10px] font-black uppercase tracking-widest opacity-20 hover:opacity-100 transition-opacity"
          >
            {t('common.close') || '关闭'}
          </button>
        </div>
      </div>
    </>,
    document.body
  );
};

export default NotificationCenter;
