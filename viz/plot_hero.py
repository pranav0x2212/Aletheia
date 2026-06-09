#!/usr/bin/env python3
"""
Hero plot for working-set sweep experiment - CLEAN VERSION.

Creates a simple, clear storytelling figure showing how prefetching masks
true memory hierarchy latency, and how pointer chasing reveals it.

Output:
    results/working_set_hero_plot.png
"""

import json
from collections import defaultdict
from pathlib import Path
from typing import Dict

import matplotlib.pyplot as plt
import numpy as np


class HeroPlot:
    """Generate clean, minimal hero plot."""

    # Cache boundaries in KB
    CACHE_BOUNDARIES_KB = {
        'L1': 32,
        'L2': 256,
        'L3': 8192,
    }

    @staticmethod
    def read_data(filepath: str, mode: str = 'cpu') -> Dict[int, list]:
        """Read JSONL and return {size_bytes: [latencies]}."""
        data_by_size = defaultdict(list)
        
        with open(filepath, 'r') as f:
            for line in f:
                if not line.strip():
                    continue
                rec = json.loads(line)
                if rec.get('mode') == mode:
                    # Try to get working set size (new format: bytes, old format: MB)
                    size_bytes = rec.get('working_set_bytes')
                    if size_bytes is None:
                        # Fallback for old format
                        size_mb = rec.get('dataset_size_mb')
                        if size_mb:
                            size_bytes = size_mb * 1024 * 1024
                    
                    if size_bytes is None:
                        continue
                    
                    # Try to get latency directly (new format for working-set sweep)
                    latency_ns = rec.get('latency_ns_per_access')
                    
                    # Fallback: reconstruct from runtime_ms and operations
                    if latency_ns is None:
                        runtime_ms = rec.get('runtime_ms')
                        operations = rec.get('operations')
                        if runtime_ms is not None and operations and operations > 0:
                            latency_ns = (runtime_ms * 1_000_000) / operations
                    
                    # Only include valid measurements
                    if latency_ns is not None and latency_ns > 0:
                        data_by_size[size_bytes].append(latency_ns)
        
        return data_by_size

    @staticmethod
    def format_size(size_bytes: int) -> str:
        """Format byte size with appropriate units."""
        if size_bytes < 1024:
            return f"{size_bytes}B"
        elif size_bytes < 1024 * 1024:
            kb = size_bytes // 1024
            return f"{kb}KB"
        else:
            mb = size_bytes // (1024 * 1024)
            return f"{mb}MB"

    @classmethod
    def create_plot(cls, seq_data: Dict, rand_data: Dict, ptr_data: Dict, output_path: str) -> None:
        """Create clean, minimal hero plot."""
        
        # Get sorted sizes and compute medians
        seq_sizes_bytes = sorted(seq_data.keys())
        seq_medians = [np.median(seq_data[s]) for s in seq_sizes_bytes]

        rand_sizes_bytes = sorted(rand_data.keys())
        rand_medians = [np.median(rand_data[s]) for s in rand_sizes_bytes]
        
        ptr_sizes_bytes = sorted(ptr_data.keys())
        ptr_medians = [np.median(ptr_data[s]) for s in ptr_sizes_bytes]
        
        # Create figure
        fig, ax = plt.subplots(figsize=(14, 8))
        
        # Plot lines - clean and minimal
        ax.loglog(seq_sizes_bytes, seq_medians,
                 linestyle='--', linewidth=2.5, marker='o', markersize=8,
                 label='Sequential Scan', color='#1f77b4', alpha=0.75,
                 markerfacecolor='white', markeredgewidth=2, zorder=5)
        
        ax.loglog(rand_sizes_bytes, rand_medians,
                  linestyle='-', linewidth=2.8, marker='^', markersize=8,
                  label='Random Access', color='#2ca02c', alpha=0.85,
                  markerfacecolor='white', markeredgewidth=2, zorder=4)
        
        ax.loglog(ptr_sizes_bytes, ptr_medians,
                 linestyle='-', linewidth=3.5, marker='s', markersize=8,
                 label='Pointer Chasing', color='#ff7f0e', alpha=0.9,
                 markerfacecolor='white', markeredgewidth=2, zorder=5)
        
        # Set log base 2 for x-axis (cache hierarchy scaling)
        ax.set_xscale('log', base=2)
        ax.set_yscale('log', base=10)
        
        # Minimal grid
        ax.grid(True, which='major', alpha=0.2, linestyle='-', linewidth=0.8)
        ax.grid(True, which='minor', alpha=0.08, linestyle=':', linewidth=0.4)
        ax.set_axisbelow(True)
        
        # Custom x-axis labels - show B/KB/MB properly
        x_ticks = []
        x_labels = []
        for size_bytes in sorted(set(seq_sizes_bytes + ptr_sizes_bytes)):
            x_ticks.append(size_bytes)
            x_labels.append(cls.format_size(size_bytes))
        
        ax.set_xticks(x_ticks)
        ax.set_xticklabels(x_labels, fontsize=10)
        
        # Add subtle cache boundary lines (convert KB to bytes for comparison)
        for cache_name, boundary_kb in cls.CACHE_BOUNDARIES_KB.items():
            boundary_bytes = boundary_kb * 1024
            ax.axvline(boundary_bytes, linestyle=':', linewidth=1.5,
                      color='gray', alpha=0.3, zorder=1)
            # Small label at top
            ax.text(boundary_bytes, ax.get_ylim()[1] * 0.8,
                   cache_name, fontsize=9, color='gray', alpha=0.6,
                   ha='center', va='top', fontweight='bold')
        
        # Minimal annotations
        # Sequential annotation
        if seq_medians:
            mid_idx = len(seq_medians) // 2
            ax.text(seq_sizes_bytes[mid_idx], seq_medians[mid_idx] * 0.4,
                   'prefetching', fontsize=9, color='#1f77b4', alpha=0.6,
                   style='italic', ha='center')
            
        if rand_medians:
            mid_idx = len(rand_medians) // 2
            ax.text(4 * 1024 * 1024, 6.0, 'MLP Survives', fontsize=9, color='#2ca02c', alpha=0.7, style='italic', ha='center')
        
        # Pointer annotation
        if ptr_medians:
            mid_idx = len(ptr_medians) // 2
            ax.text(ptr_sizes_bytes[mid_idx], ptr_medians[mid_idx] * 2.5,
                   'true latency', fontsize=9, color='#ff7f0e', alpha=0.7,
                   style='italic', ha='center')
        
        # Labels and title
        ax.set_xlabel('Working Set Size', fontsize=12, fontweight='bold')
        ax.set_ylabel('Latency (ns/access)', fontsize=12, fontweight='bold')
        ax.set_title('Memory Access Patterns Reveal Hidden Latency\n' 'Why the CPU helps some workloads more than others',
                    fontsize=14, fontweight='bold', pad=15)
        
        # Clean legend
        ax.legend(fontsize=11, loc='upper left', framealpha=0.95,
                 edgecolor='gray', fancybox=False)
        
        plt.tight_layout()
        plt.savefig(output_path, dpi=300, bbox_inches='tight', facecolor='white')
        print(f"✓ Saved: {output_path}")
        plt.close()


def main():
    """Main entry point."""
    results_dir = Path('results')
    seq_file = results_dir / 'working_set_sweep_sequential.jsonl'
    rand_file = results_dir / 'working_set_sweep_random.jsonl'
    ptr_file = results_dir / 'working_set_sweep_pointer.jsonl'
    
    if not seq_file.exists() or not rand_file.exists() or not ptr_file.exists():
        print("Error: Required JSONL files not found")
        return
    
    print("Reading working-set sweep results...")
    
    plot = HeroPlot()
    seq_data = plot.read_data(str(seq_file))
    rand_data = plot.read_data(str(rand_file))
    ptr_data = plot.read_data(str(ptr_file))
    
    print(f"  Sequential: {len(seq_data)} working set sizes")
    for size in sorted(seq_data.keys()):
        print(f"    {plot.format_size(size):>10} - {len(seq_data[size]):2d} runs, median={np.median(seq_data[size]):7.2f} ns")
    
    print(f"  Pointer: {len(ptr_data)} working set sizes")
    for size in sorted(ptr_data.keys()):
        print(f"    {plot.format_size(size):>10} - {len(ptr_data[size]):2d} runs, median={np.median(ptr_data[size]):7.2f} ns")
    
    print("\nGenerating hero plot...")
    plot.create_plot(seq_data, rand_data,ptr_data,
                    str(results_dir / 'working_set_hero_plot_v2.png'))
    
    print("\n" + "="*70)
    print("✓ Hero plot complete!")
    print("="*70)


if __name__ == '__main__':
    main()

