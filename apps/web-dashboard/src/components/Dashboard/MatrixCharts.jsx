import React from 'react';
import { 
  AreaChart, 
  Area, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell
} from 'recharts';
import { useTranslation } from 'react-i18next';
import { TriangleAlert, History } from 'lucide-react';
import { clsx } from 'clsx';

// 为不同模型定义 Matrix 配色
const MODEL_COLORS = {
  gemini: '#3b82f6', // 蓝
  claude: '#f59e0b', // 琥珀
  gpt: '#10b981',    // 绿
  default: '#64748b' // 灰
};

const getModelColor = (name = "") => {
  const n = name.toLowerCase();
  if (n.includes('gemini')) return MODEL_COLORS.gemini;
  if (n.includes('claude')) return MODEL_COLORS.claude;
  if (n.includes('gpt')) return MODEL_COLORS.gpt;
  return MODEL_COLORS.default;
};

// 1. Traffic Stream Area Chart
export const TrafficStreamChart = ({ data = [] }) => {
  const { t } = useTranslation();
  // 获取所有出现的模型 Key (排除 period)
  const modelKeys = Array.from(new Set(
    data.flatMap(d => Object.keys(d).filter(k => k !== 'period'))
  ));

  return (
    <div className="glass-card spotlight-card p-6 rounded-2xl h-[340px] flex flex-col transition-all duration-300 relative overflow-hidden">
      <div className="grain-overlay" />
      <div className="flex justify-between items-center mb-6 relative z-10">
        <h3 className="text-foreground/30 font-bold text-[10px] uppercase tracking-[0.2em] flex items-center gap-2">
          <History size={12} /> {t('dashboard.trafficFlowLine')}
        </h3>
        <div className="flex gap-3">
          {modelKeys.slice(0, 3).map(k => (
            <div key={k} className="flex items-center gap-1.5">
              <div className="w-2 h-2 rounded-full" style={{ backgroundColor: getModelColor(k) }} />
              <span className="text-[9px] uppercase text-foreground/40 font-bold">{k}</span>
            </div>
          ))}
        </div>
      </div>
      
      <div className="flex-1 w-full min-h-0 relative z-10">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data} margin={{ top: 0, right: 0, left: -20, bottom: 0 }}>
            <defs>
              {modelKeys.map(k => (
                <linearGradient key={`grad-${k}`} id={`grad-${k}`} x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor={getModelColor(k)} stopOpacity={0.3}/>
                  <stop offset="95%" stopColor={getModelColor(k)} stopOpacity={0}/>
                </linearGradient>
              ))}
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="var(--glass-border)" vertical={false} />
            <XAxis 
              dataKey="period" 
              hide={true}
            />
            <YAxis 
              stroke="var(--text-dim)" 
              fontSize={10}
              tick={{ fill: 'var(--text-muted)' }}
              tickFormatter={(v) => v >= 1000 ? `${(v/1000).toFixed(1)}k` : v}
            />
            <Tooltip 
              contentStyle={{ backgroundColor: 'var(--card-bg)', border: '1px solid var(--glass-border)', borderRadius: '12px', backdropBlur: '12px' }}
              itemStyle={{ fontSize: '11px', fontWeight: 'bold' }}
              labelStyle={{ color: 'var(--text-muted)', marginBottom: '4px', fontSize: '10px' }}
            />
            {modelKeys.map(k => (
              <Area 
                key={k}
                type="monotone" 
                dataKey={k} 
                stackId="1" 
                stroke={getModelColor(k)} 
                fill={`url(#grad-${k})`} 
                strokeWidth={2}
              />
            ))}
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
};

// 2. Model Distribution Donut
export const ModelDistributionChart = ({ stats = [] }) => {
  const { t } = useTranslation();
  const chartData = stats.map(s => ({
    name: s.model,
    value: s.tokens
  }));

  return (
    <div className="glass-card spotlight-card p-6 rounded-2xl h-[340px] flex flex-col transition-all duration-300 relative overflow-hidden">
      <div className="grain-overlay" />
      <h3 className="text-foreground/30 font-bold text-[10px] uppercase tracking-[0.2em] mb-4 relative z-10">{t('dashboard.modelDistribution')}</h3>
      <div className="flex-1 flex items-center justify-center relative z-10">
        <ResponsiveContainer width="100%" height="100%">
          <PieChart>
            <Pie
              data={chartData}
              innerRadius={60}
              outerRadius={90}
              paddingAngle={5}
              dataKey="value"
            >
              {chartData.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={getModelColor(entry.name)} stroke="transparent" />
              ))}
            </Pie>
            <Tooltip 
              contentStyle={{ backgroundColor: 'var(--card-bg)', border: '1px solid var(--glass-border)', borderRadius: '12px' }}
              itemStyle={{ fontSize: '11px' }}
            />
          </PieChart>
        </ResponsiveContainer>
      </div>
      <div className="mt-4 space-y-1.5 relative z-10">
        {stats.slice(0, 3).map(s => (
          <div key={s.model} className="flex justify-between items-center text-[10px]">
            <span className="text-foreground/40 uppercase font-mono">{s.model}</span>
            <span className="text-foreground/60 font-bold">{(s.tokens/1000).toFixed(1)}k</span>
          </div>
        ))}
      </div>
    </div>
  );
};

// 3. Quota Health Progress
export const QuotaHealthCard = ({ accounts = [] }) => {
  const { t } = useTranslation();
  const total = accounts.length || 1;
  const activeCount = accounts.filter(a => 
    (a.status === 'Active' || a.status === 'active') && 
    !a.is_proxy_disabled && 
    !a.quota?.is_forbidden
  ).length;
  const limitedCount = accounts.filter(a => 
    a.status?.includes('Limit') || 
    (a.is_proxy_disabled && !a.quota?.is_forbidden)
  ).length;
  const errorCount = accounts.filter(a => 
    a.status?.includes('Error') || 
    a.status?.includes('Invalid') || 
    a.quota?.is_forbidden
  ).length;

  const getPercent = (count) => (count / total) * 100;

  return (
    <div className="glass-card spotlight-card p-6 rounded-2xl h-[280px] transition-all duration-300 relative overflow-hidden">
      <div className="grain-overlay" />
      <h3 className="text-foreground/30 font-bold text-[10px] uppercase tracking-[0.2em] mb-6 relative z-10">{t('dashboard.quotaHealth')}</h3>
      
      <div className="text-3xl font-black text-foreground tracking-tighter mb-4 relative z-10">
        {activeCount}<span className="text-xs ml-1 text-foreground/30 font-bold uppercase">{t('dashboard.ready')}</span>
      </div>

      <div className="h-4 w-full bg-foreground/5 rounded-full overflow-hidden flex relative z-10">
        <div style={{ width: `${getPercent(activeCount)}%` }} className="h-full bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.3)]" />
        <div style={{ width: `${getPercent(limitedCount)}%` }} className="h-full bg-amber-500/60" />
        <div style={{ width: `${getPercent(errorCount)}%` }} className="h-full bg-red-500/60" />
      </div>

      <div className="grid grid-cols-3 gap-2 mt-6 relative z-10">
        <div>
           <div className="text-[10px] text-foreground/20 uppercase font-bold mb-1">{t('dashboard.active')}</div>
           <div className="h-0.5 w-full bg-blue-500/30" />
           <div className="text-sm font-bold mt-1 text-foreground/80">{activeCount}</div>
        </div>
        <div>
           <div className="text-[10px] text-foreground/20 uppercase font-bold mb-1">{t('dashboard.limited')}</div>
           <div className="h-0.5 w-full bg-amber-500/30" />
           <div className="text-sm font-bold mt-1 text-foreground/80">{limitedCount}</div>
        </div>
        <div>
           <div className="text-[10px] text-foreground/20 uppercase font-bold mb-1">{t('dashboard.error')}</div>
           <div className="h-0.5 w-full bg-red-500/30" />
           <div className="text-sm font-bold mt-1 text-foreground/80">{errorCount}</div>
        </div>
      </div>
    </div>
  );
};

// 4. Fault Hotspots List
export const FaultHotspotsList = ({ faults = [] }) => {
  const { t } = useTranslation();
  return (
    <div className="glass-card spotlight-card p-6 rounded-2xl h-[280px] flex flex-col transition-all duration-300 relative overflow-hidden">
      <div className="grain-overlay" />
      <h3 className="text-foreground/30 font-bold text-[10px] uppercase tracking-[0.2em] mb-6 flex items-center gap-2 relative z-10">
        <TriangleAlert size={12} className="text-amber-500" /> {t('dashboard.faultHotspots')}
      </h3>
      <div className="flex-1 overflow-y-auto space-y-3 pr-2 scrollbar-none relative z-10">
        {faults.length === 0 ? (
           <div className="h-full flex items-center justify-center text-[10px] text-foreground/10 italic">
             {t('dashboard.noAnomalies')}
           </div>
        ) : (
          faults.map((f, i) => (
            <div key={i} className="bg-foreground/[0.02] border border-glass-border p-2 rounded-lg flex justify-between items-center group hover:border-red-500/20 transition-colors">
              <div className="flex flex-col">
                <span className="text-[10px] font-mono font-bold text-red-500/80 uppercase tracking-tighter">
                  {f.status} {f.error || 'System Timeout'}
                </span>
                <span className="text-[9px] text-foreground/20 font-mono mt-0.5">
                  ID: {f.id?.slice(0, 8)} // {f.model || 'Unknown'}
                </span>
              </div>
              <div className="text-[8px] text-foreground/10 font-mono">
                {new Date(f.timestamp * 1000).toLocaleTimeString()}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

// 5. Account Load Card (Simple Bar)
export const AccountLoadCard = ({ stats = [] }) => {
  const { t } = useTranslation();
  return (
    <div className="glass-card spotlight-card p-6 rounded-2xl h-[280px] transition-all duration-300 relative overflow-hidden">
      <div className="grain-overlay" />
      <h3 className="text-foreground/30 font-bold text-[10px] uppercase tracking-[0.2em] mb-6 relative z-10">{t('dashboard.topAccountLoad')}</h3>
      <div className="space-y-4 relative z-10">
        {stats.slice(0, 4).map((s, i) => (
          <div key={i} className="space-y-1">
            <div className="flex justify-between text-[10px] uppercase font-mono tracking-tighter">
               <span className="text-foreground/40 truncate w-32 font-bold">{s.account}</span>
               <span className="text-foreground/60 font-bold">{(s.tokens/1000).toFixed(1)}k</span>
            </div>
            <div className="h-1 w-full bg-foreground/5 rounded-full overflow-hidden">
               <div style={{ width: `${Math.min(100, (s.tokens / stats[0].tokens) * 100)}%` }} className="h-full bg-foreground/10 group-hover:bg-blue-500/50 transition-all duration-500" />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
