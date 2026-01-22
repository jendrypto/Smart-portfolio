import usePortfolioStore from '../store/portfolio';
import SummaryPanel from './SummaryPanel';
import InsightCards from './InsightCards';
import PositionsTable from './PositionsTable';
import EmptyStateView from './EmptyStateView';
import DemoBanner from './DemoBanner';

function HoldingsTab() {
  const { positions, summary, snapshots, isLoading } = usePortfolioStore();

  const isEmpty = positions.length === 0;
  const hasDemoPositions = positions.some(
    p => p.position.user_tags?.includes('demo')
  );

  if (isLoading && isEmpty) {
    return <div className="loading">Loading holdings...</div>;
  }

  if (isEmpty) {
    return <EmptyStateView />;
  }

  return (
    <div className="holdings-tab">
      {/* Demo Banner - show only if demo positions present */}
      {hasDemoPositions && <DemoBanner />}

      {/* Insight Cards */}
      <InsightCards />

      {/* Summary Panel with Chart and Stats */}
      {summary && <SummaryPanel summary={summary} snapshots={snapshots} />}

      {/* Positions Table */}
      <PositionsTable />
    </div>
  );
}

export default HoldingsTab;
