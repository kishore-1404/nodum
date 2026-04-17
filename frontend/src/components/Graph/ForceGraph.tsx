import { useEffect, useRef, useState, useCallback } from 'react';
import * as d3 from 'd3';
import type { GraphData, GraphNode, KnowledgeEdge } from '../../types';

interface Props {
  data: GraphData;
  width: number;
  height: number;
  onNodeClick?: (node: GraphNode) => void;
  onNodeHover?: (node: GraphNode | null) => void;
  selectedNodeId?: string | null;
}

interface SimNode extends d3.SimulationNodeDatum, GraphNode {}
interface SimLink extends d3.SimulationLinkDatum<SimNode> {
  id: string;
  relation_type: string;
  weight: number;
}

const NODE_TYPE_SHAPES: Record<string, string> = {
  concept: 'circle',
  fact: 'square',
  principle: 'diamond',
  example: 'triangle',
  question: 'star',
  insight: 'hexagon',
  prior_knowledge: 'circle',
};

const RELATION_COLORS: Record<string, string> = {
  builds_on: '#6366f1',
  contradicts: '#ef4444',
  is_example_of: '#10b981',
  requires: '#f59e0b',
  extends: '#3b82f6',
  is_part_of: '#8b5cf6',
  analogous_to: '#ec4899',
  causes: '#14b8a6',
  derived_from: '#6366f1',
  related_to: '#9a917d',
  supports: '#3b82f6',
};

export function ForceGraph({ data, width, height, onNodeClick, onNodeHover, selectedNodeId }: Props) {
  const svgRef = useRef<SVGSVGElement>(null);
  const simulationRef = useRef<d3.Simulation<SimNode, SimLink>>();

  useEffect(() => {
    if (!svgRef.current || !data.nodes.length) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    // Prepare data
    const nodes: SimNode[] = data.nodes.map(n => ({ ...n }));
    const nodeMap = new Map(nodes.map(n => [n.id, n]));
    const links: SimLink[] = data.edges
      .filter(e => nodeMap.has(e.source) && nodeMap.has(e.target))
      .map(e => ({
        ...e,
        source: e.source,
        target: e.target,
      }));

    // Create container with zoom
    const g = svg.append('g');
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => g.attr('transform', event.transform));
    svg.call(zoom);

    // Arrow markers for directed edges
    const defs = svg.append('defs');
    Object.entries(RELATION_COLORS).forEach(([type, color]) => {
      defs.append('marker')
        .attr('id', `arrow-${type}`)
        .attr('viewBox', '0 -5 10 10')
        .attr('refX', 20)
        .attr('refY', 0)
        .attr('markerWidth', 6)
        .attr('markerHeight', 6)
        .attr('orient', 'auto')
        .append('path')
        .attr('d', 'M0,-5L10,0L0,5')
        .attr('fill', color)
        .attr('opacity', 0.5);
    });

    // Glow filter
    const filter = defs.append('filter').attr('id', 'glow');
    filter.append('feGaussianBlur').attr('stdDeviation', '3').attr('result', 'coloredBlur');
    const merge = filter.append('feMerge');
    merge.append('feMergeNode').attr('in', 'coloredBlur');
    merge.append('feMergeNode').attr('in', 'SourceGraphic');

    // Simulation
    const simulation = d3.forceSimulation<SimNode>(nodes)
      .force('link', d3.forceLink<SimNode, SimLink>(links)
        .id(d => d.id)
        .distance(120)
        .strength(d => d.weight * 0.3))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30))
      .force('x', d3.forceX(width / 2).strength(0.05))
      .force('y', d3.forceY(height / 2).strength(0.05));

    simulationRef.current = simulation;

    // Draw edges
    const link = g.append('g')
      .selectAll('line')
      .data(links)
      .join('line')
      .attr('class', 'graph-edge')
      .attr('stroke', d => RELATION_COLORS[d.relation_type] || '#9a917d')
      .attr('stroke-width', d => Math.max(1, d.weight * 1.5))
      .attr('marker-end', d => `url(#arrow-${d.relation_type})`);

    // Edge labels
    const edgeLabels = g.append('g')
      .selectAll('text')
      .data(links)
      .join('text')
      .attr('font-size', '8px')
      .attr('fill', 'var(--text-muted)')
      .attr('text-anchor', 'middle')
      .attr('opacity', 0)
      .text(d => d.relation_type.replace(/_/g, ' '));

    // Draw nodes
    const nodeGroup = g.append('g')
      .selectAll('g')
      .data(nodes)
      .join('g')
      .attr('class', 'graph-node')
      .call(d3.drag<SVGGElement, SimNode>()
        .on('start', (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on('drag', (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on('end', (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        }));

    // Node circles
    nodeGroup.append('circle')
      .attr('r', d => 6 + Math.min(d.connection_count * 2, 14))
      .attr('fill', d => d.book_color || '#6366f1')
      .attr('opacity', d => 0.4 + d.confidence_score * 0.6)
      .attr('stroke', d => d.id === selectedNodeId ? 'var(--accent)' : 'transparent')
      .attr('stroke-width', d => d.id === selectedNodeId ? 3 : 0)
      .attr('filter', d => d.id === selectedNodeId ? 'url(#glow)' : null);

    // Node labels
    nodeGroup.append('text')
      .text(d => d.label.length > 24 ? d.label.slice(0, 22) + '…' : d.label)
      .attr('font-size', '10px')
      .attr('font-family', '"DM Sans", sans-serif')
      .attr('font-weight', '500')
      .attr('fill', 'var(--text-primary)')
      .attr('dy', d => -(10 + Math.min(d.connection_count * 2, 14)))
      .attr('text-anchor', 'middle')
      .attr('opacity', 0.8);

    // Interactions
    nodeGroup
      .on('click', (_, d) => onNodeClick?.(d))
      .on('mouseenter', (event, d) => {
        onNodeHover?.(d);
        d3.select(event.currentTarget).select('circle')
          .transition().duration(200)
          .attr('filter', 'url(#glow)');

        // Highlight connected edges
        link.attr('stroke-opacity', l =>
          (l.source as SimNode).id === d.id || (l.target as SimNode).id === d.id ? 1 : 0.1
        );
        edgeLabels.attr('opacity', l =>
          (l.source as SimNode).id === d.id || (l.target as SimNode).id === d.id ? 1 : 0
        );
        nodeGroup.select('circle').attr('opacity', n =>
          n.id === d.id ||
          links.some(l =>
            ((l.source as SimNode).id === d.id && (l.target as SimNode).id === n.id) ||
            ((l.target as SimNode).id === d.id && (l.source as SimNode).id === n.id)
          ) ? 1 : 0.15
        );
      })
      .on('mouseleave', (event, d) => {
        onNodeHover?.(null);
        d3.select(event.currentTarget).select('circle')
          .transition().duration(200)
          .attr('filter', d.id === selectedNodeId ? 'url(#glow)' : null);

        link.attr('stroke-opacity', 0.4);
        edgeLabels.attr('opacity', 0);
        nodeGroup.select('circle').attr('opacity', n => 0.4 + n.confidence_score * 0.6);
      });

    // Tick
    simulation.on('tick', () => {
      link
        .attr('x1', d => (d.source as SimNode).x!)
        .attr('y1', d => (d.source as SimNode).y!)
        .attr('x2', d => (d.target as SimNode).x!)
        .attr('y2', d => (d.target as SimNode).y!);

      edgeLabels
        .attr('x', d => ((d.source as SimNode).x! + (d.target as SimNode).x!) / 2)
        .attr('y', d => ((d.source as SimNode).y! + (d.target as SimNode).y!) / 2);

      nodeGroup.attr('transform', d => `translate(${d.x},${d.y})`);
    });

    return () => {
      simulation.stop();
    };
  }, [data, width, height, selectedNodeId]);

  return (
    <svg
      ref={svgRef}
      width={width}
      height={height}
      style={{ background: 'transparent' }}
    />
  );
}
