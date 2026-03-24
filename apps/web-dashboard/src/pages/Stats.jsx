import React, { useEffect, useMemo } from 'react';
import { useSpotlight } from '../hooks/useSpotlight';
import { useTranslation } from 'react-i18next';
import { 
  AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, 
  BarChart, Bar, Cell, PieChart, Pie, Legend, LabelList
} from 'recharts';
import { 
  Zap, Users, Cpu, Activity, TrendingUp, ChevronRight, 
  Clock, Hash, ArrowUpRight, BarChart3, CalendarDays
} from 'lucide-react';
import { useStatsStore } from '../store/useStatsStore';

const COLORS = ['#FF0055', '#FF8800', '#00EEFF', '#8844FF', '#FFCC00', '#00FF88'];

const formatTokens = (val) => {
  if (val >= 1000000) return (val / 1000000).toFixed(1) + 'M';
  if (val >= 1000) return (val / 1000).toFixed(1) + 'K';
  return val;
};

// -----------------------------------------------------
// 微组件：顶部卡片
// -----------------------------------------------------
const StatsCard = ({ title, value, icon: Icon, subtext }) => (
  <div className="bg-foreground/[0.03] spotlight-card border border-glass-border rounded-2xl p-6 hover:bg-foreground/[0.05] transition-all group relative overflow-hidden">
    <div className="grain-overlay" />
    <div className="absolute top-0 right-0 w-24 h-24 bg-foreground/5 blur-3xl rounded-full -mr-10 -mt-10 group-hover:bg-foreground/10 transition-all"></div>
    <div className="flex justify-between items-start mb-4 relative z-10">
      <div className="p-3 bg-foreground/5 rounded-xl border border-glass-border">
        <Icon size={20} className="text-foreground/70" />
      </div>
    </div>
    <div className="relative z-10">
      <h3 className="text-foreground/40 text-xs font-bold tracking-wider mb-1 uppercase">{title}</h3>
      <p className="text-2xl font-black text-foreground font-mono">{value}</p>
      {subtext && <p className="text-foreground/30 text-xs mt-2 uppercase">{subtext}</p>}
    </div>
  </div>
);

// -----------------------------------------------------
// 微组件：区块标题
// -----------------------------------------------------
const SectionHeader = ({ title, icon: Icon, subtitle, action }) => (
  <div className="flex items-center justify-between mb-8">
    <div className="flex items-center space-x-4">
      <div className="p-3 bg-foreground/5 rounded-2xl border border-glass-border shadow-inner">
        <Icon size={24} className="text-foreground/80" />
      </div>
      <div>
        <h2 className="text-xl font-black text-foreground tracking-tight uppercase italic">{title}</h2>
        <p className="text-foreground/30 text-sm font-medium uppercase">{subtitle}</p>
      </div>
    </div>
    {action && <div>{action}</div>}
  </div>
);

// -----------------------------------------------------
// 页面主入口
// -----------------------------------------------------
const Stats = () => {
  const { t } = useTranslation();
  const spotlightRef = useSpotlight();
  const spotlightChartsRef = useSpotlight();
  const { 
    timeRange, setTimeRange, 
    summary, trendData, modelStats, accountStats, modelTrends, 
    fetchStats, isLoading 
  } = useStatsStore();

  useEffect(() => {
    fetchStats();
    const interval = setInterval(fetchStats, 60000);
    return () => clearInterval(interval);
  }, [fetchStats]);

  // 格式化时间轴 (如果是天级则取后半部分日期, 如果是小时则取小时)
  const formattedModelTrends = useMemo(() => {
    return modelTrends.map(point => {
      const isDaily = point.period.length <= 10;
      return {
        ...point,
        displayTime: isDaily ? point.period.substring(5) : point.period.split(' ')[1]
      };
    });
  }, [modelTrends]);

  const activeModels = useMemo(() => {
    const models = new Set();
    modelTrends.forEach(p => {
      Object.keys(p).forEach(k => {
        if (k !== 'period' && k !== 'displayTime') models.add(k);
      });
    });
    return Array.from(models);
  }, [modelTrends]);

  if (isLoading && !summary) {
    return (
      <div className="min-h-[60vh] flex items-center justify-center">
        <div className="flex flex-col items-center space-y-4">
          <div className="loading loading-spinner loading-lg text-foreground/50"></div>
          <p className="text-foreground/30 font-mono text-sm animate-pulse tracking-widest uppercase">{t('stats.loading')}</p>
        </div>
      </div>
    );
  }

  const getTimeRangeLabel = () => {
    switch (timeRange) {
      case '24h': return t('stats.range24h');
      case '7d': return t('stats.range7d');
      case '30d': return t('stats.range30d');
      case 'all': return t('stats.rangeAll');
      default: return timeRange;
    }
  };

  return (
    <div ref={spotlightRef} className="p-8 space-y-12 fade-in spotlight-group">
      
      {/* 标题 & 时间控制台 */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center border-b border-glass-border pb-6">
        <div>
          <h1 className="text-3xl font-black text-foreground italic tracking-tighter uppercase">{t('stats.title')}</h1>
          <p className="text-[10px] font-black text-foreground/20 uppercase tracking-[0.4em] mt-1">{t('stats.subtitle')}</p>
        </div>
        <div className="flex items-center space-x-2 mt-4 md:mt-0 bg-foreground/5 p-1 rounded-xl border border-glass-border">
          {[
            { value: '24h', label: '1D' },
            { value: '7d', label: '7D' },
            { value: '30d', label: '30D' },
            { value: 'all', label: 'ALL' }
          ].map(range => (
            <button
              key={range.value}
              onClick={() => setTimeRange(range.value)}
              className={`px-4 py-2 text-[10px] font-black rounded-lg transition-all uppercase tracking-widest ${
                timeRange === range.value 
                  ? 'bg-foreground text-background shadow-[0_0_15px_rgba(255,255,255,0.3)]' 
                  : 'text-foreground/50 hover:text-foreground hover:bg-foreground/10'
              }`}
            >
              {range.label}
            </button>
          ))}
        </div>
      </div>

      {/* 概览卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatsCard 
          title={t('stats.totalTokens')} 
          value={formatTokens(summary?.total_tokens || 0)} 
          icon={Zap} 
          subtext={`${t('stats.input')}: ${formatTokens(summary?.total_input_tokens || 0)} / ${t('stats.output')}: ${formatTokens(summary?.total_output_tokens || 0)}`}
        />
        <StatsCard 
          title={t('stats.totalRequests')} 
          value={summary?.total_requests || 0} 
          icon={Activity} 
          subtext={t('stats.requestsDesc')}
        />
        <StatsCard 
          title={t('stats.uniqueAccounts')} 
          value={summary?.unique_accounts || 0} 
          icon={Users} 
          subtext={t('stats.accountsDesc')}
        />
        <StatsCard 
          title={t('stats.currentRange')} 
          value={getTimeRangeLabel()} 
          icon={CalendarDays} 
          subtext={t('stats.rangeDesc')}
        />
      </div>

      {/* 趋势部分：堆叠面积图 */}
      <div ref={spotlightChartsRef} className="bg-foreground/[0.02] border border-glass-border rounded-3xl p-8 spotlight-card relative overflow-hidden spotlight-group">
        <div className="grain-overlay" />
        <SectionHeader 
          title={t('stats.trendTitle', { range: getTimeRangeLabel() })}
          subtitle={t('stats.trendSub')} 
          icon={BarChart3}
        />
        <div className="h-[350px] w-full flex items-center justify-center">
          {formattedModelTrends.length > 0 ? (
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={formattedModelTrends}>
                <defs>
                  {activeModels.map((m, i) => (
                    <linearGradient key={m} id={`color${m}`} x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor={COLORS[i % COLORS.length]} stopOpacity={0.3}/>
                      <stop offset="95%" stopColor={COLORS[i % COLORS.length]} stopOpacity={0}/>
                    </linearGradient>
                  ))}
                </defs>
                <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="var(--glass-border-color)" />
                <XAxis 
                  dataKey="displayTime" 
                  axisLine={false} 
                  tickLine={false} 
                  tick={{fill: 'currentColor', opacity: 0.6, fontSize: 10, fontWeight: 'bold'}} className="text-foreground"
                />
                <YAxis 
                  axisLine={false} 
                  tickLine={false} 
                  tick={{fill: 'currentColor', opacity: 0.6, fontSize: 10, fontWeight: 'bold'}} className="text-foreground"
                  tickFormatter={formatTokens}
                />
                <Tooltip 
                  contentStyle={{ backgroundColor: 'var(--background)', border: '1px solid var(--glass-border-color)', borderRadius: '12px' }}
                  itemStyle={{ fontSize: '12px', fontWeight: 'bold' }}
                />
                {activeModels.map((m, i) => (
                  <Area 
                    key={m} 
                    type="monotone" 
                    dataKey={m} 
                    stackId="1" 
                    stroke={COLORS[i % COLORS.length]} 
                    fillOpacity={1} 
                    fill={`url(#color${m})`} 
                    strokeWidth={2}
                  />
                ))}
              </AreaChart>
            </ResponsiveContainer>
          ) : (
            <div className="text-foreground/10 font-black uppercase tracking-[0.3em] text-xs italic">
              {t('stats.noData')}
            </div>
          )}
        </div>
      </div>

      {/* 中间层：模型消费柱状图 (双排并列) */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div className="bg-foreground/[0.02] spotlight-card border border-glass-border rounded-3xl p-8 relative overflow-hidden">
          <div className="grain-overlay" />
          <SectionHeader 
            title={t('stats.compareTitle')} 
            subtitle={t('stats.compareSub')} 
            icon={Cpu}
          />
          <div className="h-[300px] w-full flex items-center justify-center">
            {modelStats.length > 0 ? (
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={modelStats} layout="vertical" margin={{ top: 0, right: 60, left: 40, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" horizontal={true} vertical={false} stroke="var(--glass-border-color)" />
                  <XAxis type="number" hide />
                  <YAxis dataKey="model" type="category" axisLine={false} tickLine={false} tick={{fill: 'currentColor', opacity: 0.7, fontSize: 10, fontWeight: 'bold'}} className="text-foreground" width={120} />
                  <Tooltip 
                    cursor={{fill: 'var(--foreground)', opacity: 0.05}}
                    contentStyle={{ backgroundColor: 'var(--background)', border: '1px solid var(--glass-border-color)', borderRadius: '12px' }} 
                    itemStyle={{ fontSize: '12px', fontWeight: 'bold' }}
                  />
                  <Bar dataKey="tokens" radius={[0, 4, 4, 0]} maxBarSize={16}>
                    {modelStats.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                    ))}
                    <LabelList dataKey="tokens" position="right" fill="currentColor" opacity={0.6} className="text-foreground" formatter={formatTokens} fontSize={10} fontWeight="bold" />
                  </Bar>
                </BarChart>
              </ResponsiveContainer>
            ) : (
              <div className="text-foreground/10 font-black uppercase tracking-[0.3em] text-xs italic">
                {t('stats.noData')}
              </div>
            )}
          </div>
        </div>

        <div className="bg-foreground/[0.02] spotlight-card border border-glass-border rounded-3xl p-8 relative overflow-hidden">
          <div className="grain-overlay" />
          <SectionHeader 
            title={t('stats.preferenceTitle')} 
            subtitle={t('stats.preferenceSub')} 
            icon={PieChart}
          />
          <div className="h-[300px] w-full relative flex flex-col items-center justify-center">
            {modelStats.length > 0 ? (
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={modelStats}
                    dataKey="tokens"
                    nameKey="model"
                    cx="50%"
                    cy="45%"
                    innerRadius={70}
                    outerRadius={90}
                    paddingAngle={5}
                  >
                    {modelStats.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} stroke="rgba(0,0,0,0.5)" strokeWidth={2} />
                    ))}
                  </Pie>
                  <Tooltip 
                    contentStyle={{ backgroundColor: 'var(--background)', border: '1px solid var(--glass-border-color)', borderRadius: '12px' }}
                  />
                  <Legend verticalAlign="bottom" height={36} iconType="circle" wrapperStyle={{ fontSize: '11px', fontWeight: 'bold', color: 'var(--foreground)', opacity: 0.7 }} />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div className="text-foreground/10 font-black uppercase tracking-[0.3em] text-xs italic">
                {t('stats.noData')}
              </div>
            )}
            {modelStats.length > 0 && (
              <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none pb-8">
                 <span className="text-foreground/20 text-[10px] font-black tracking-widest uppercase">{t('stats.leaderRate')}</span>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 底部排期：账号排行榜 */}
      <div className="bg-foreground/[0.02] spotlight-card border border-glass-border rounded-3xl p-8 overflow-hidden relative">
        <div className="grain-overlay" />
        <SectionHeader 
          title={t('stats.rankTitle')} 
          subtitle={t('stats.rankSub', { range: getTimeRangeLabel() })} 
          icon={Users}
        />
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="border-b border-glass-border text-[10px] font-black uppercase tracking-widest text-foreground/20">
                <th className="py-4 px-4">{t('stats.rank')}</th>
                <th className="py-4 px-4">{t('stats.rankAccount')}</th>
                <th className="py-4 px-4">{t('stats.rankTokens')}</th>
                <th className="py-4 px-4">{t('stats.rankRequests')}</th>
                <th className="py-4 px-4 text-right">{t('stats.rankStatus')}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-glass-border">
              {accountStats.map((acc, index) => (
                <tr key={acc.account} className="group hover:bg-foreground/[0.02] transition-colors">
                  <td className="py-5 px-4">
                    <span className={`flex items-center justify-center w-6 h-6 rounded-md text-[10px] font-black ${
                      index === 0 ? 'bg-amber-400/90 text-black shadow-[0_0_10px_rgba(251,191,36,0.3)]' : 
                      index === 1 ? 'bg-slate-300/80 text-black shadow-[0_0_10px_rgba(203,213,225,0.3)]' : 
                      index === 2 ? 'bg-orange-800/80 text-white shadow-[0_0_10px_rgba(154,52,18,0.3)]' : 'bg-foreground/10 text-foreground/50'
                    }`}>
                      {index + 1}
                    </span>
                  </td>
                  <td className="py-5 px-4 font-mono text-sm text-foreground/70 group-hover:text-foreground transition-colors">{acc.account}</td>
                  <td className="py-5 px-4 font-mono text-sm font-bold text-foreground/80">{formatTokens(acc.tokens)}</td>
                  <td className="py-5 px-4 font-mono text-sm text-foreground/40">{acc.requests}</td>
                  <td className="py-5 px-4 text-right">
                    <div className="inline-flex items-center text-[10px] font-bold text-emerald-400/70 bg-emerald-400/5 px-2 py-0.5 rounded border border-emerald-400/10 uppercase">
                      {t('stats.monitoring')}
                    </div>
                  </td>
                </tr>
              ))}
              {accountStats.length === 0 && (
                <tr>
                  <td colSpan="5" className="py-20 text-center text-foreground/10 font-bold tracking-widest uppercase">{t('stats.noData')}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

export default Stats;
