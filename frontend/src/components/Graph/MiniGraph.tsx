import { ForceGraph } from './ForceGraph';
import type { GraphData } from '../../types';

interface Props {
  data: GraphData;
  bookColor: string;
}

export function MiniGraph({ data, bookColor }: Props) {
  return (
    <div className="w-full h-full">
      <ForceGraph
        data={data}
        width={320}
        height={200}
      />
    </div>
  );
}
