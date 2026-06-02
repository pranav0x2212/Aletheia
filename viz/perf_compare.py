#!/usr/bin/env python3
"""
Performance comparison utility for Aletheia experiments.

Compares perf stat outputs between two experiment runs and generates
a detailed performance metrics table with derived insights.

Usage:
    python tools/perf_compare.py <file1> <file2> [--labels "Label 1" "Label 2"]

Example:
    # Generate perf stat outputs
    perf stat -o perf_sequential.txt ./target/release/aletheia-host experiment working-set-sweep --mode sequential
    perf stat -o perf_pointer.txt ./target/release/aletheia-host experiment working-set-sweep --mode pointer

    # Compare results
    python tools/perf_compare.py perf_sequential.txt perf_pointer.txt --labels "Sequential" "Pointer Chasing"

Output:
    A formatted table showing:
    - Raw metrics (cycles, instructions, cache stats, etc.)
    - Derived metrics (IPC, cache miss rate, etc.)
    - Delta ratios and percentage changes
    - Performance interpretation notes
"""

import re
import sys
from pathlib import Path
from typing import Dict, Optional, Tuple


class PerfStatParser:
    """Parse perf stat output files and extract performance metrics."""

    # Regex patterns for common perf stat metrics
    # Supports perf event suffixes (:u, :k, etc.) and alternative names (cpu-cycles)
    PATTERNS = {
        'cycles': r'([\d,]+)\s+(?:cpu-)?cycles(?::\w+)?',
        'instructions': r'([\d,]+)\s+instructions(?::\w+)?',
        'cache_references': r'([\d,]+)\s+cache-references(?::\w+)?',
        'cache_misses': r'([\d,]+)\s+cache-misses(?::\w+)?',
        'branches': r'([\d,]+)\s+branches(?::\w+)?',
        'branch_misses': r'([\d,]+)\s+branch-misses(?::\w+)?',
        'l1d_loads': r'([\d,]+)\s+L1-dcache-loads(?::\w+)?',
        'l1d_load_misses': r'([\d,]+)\s+L1-dcache-load-misses(?::\w+)?',
        'runtime': r'([\d.]+)\s+seconds time elapsed',
    }

    @staticmethod
    def parse_number(s: str) -> float:
        """Convert string with commas to number."""
        return float(s.replace(',', ''))

    @classmethod
    def parse_file(cls, filepath: str) -> Dict[str, float]:
        """Parse a perf stat output file and return metrics dict."""
        try:
            with open(filepath, 'r') as f:
                content = f.read()
        except FileNotFoundError:
            print(f"Error: File not found: {filepath}")
            sys.exit(1)

        metrics = {}
        for key, pattern in cls.PATTERNS.items():
            match = re.search(pattern, content)
            if match:
                try:
                    metrics[key] = cls.parse_number(match.group(1))
                except (ValueError, IndexError):
                    metrics[key] = None
            else:
                metrics[key] = None

        return metrics


class MetricsCalculator:
    """Calculate derived metrics from raw perf stat data."""

    @staticmethod
    def calculate_ipc(metrics: Dict) -> Optional[float]:
        """Instructions per cycle."""
        if metrics.get('instructions') and metrics.get('cycles'):
            return metrics['instructions'] / metrics['cycles']
        return None

    @staticmethod
    def calculate_cache_miss_rate(metrics: Dict) -> Optional[float]:
        """Cache miss rate as percentage."""
        if metrics.get('cache_misses') and metrics.get('cache_references'):
            return (metrics['cache_misses'] / metrics['cache_references']) * 100
        return None

    @staticmethod
    def calculate_branch_miss_rate(metrics: Dict) -> Optional[float]:
        """Branch misprediction rate as percentage."""
        if metrics.get('branch_misses') and metrics.get('branches'):
            return (metrics['branch_misses'] / metrics['branches']) * 100
        return None

    @staticmethod
    def calculate_l1d_miss_rate(metrics: Dict) -> Optional[float]:
        """L1 D-cache miss rate as percentage."""
        if metrics.get('l1d_load_misses') and metrics.get('l1d_loads'):
            return (metrics['l1d_load_misses'] / metrics['l1d_loads']) * 100
        return None

    @classmethod
    def compute_all(cls, metrics: Dict) -> Dict[str, Optional[float]]:
        """Compute all derived metrics."""
        return {
            'ipc': cls.calculate_ipc(metrics),
            'cache_miss_rate': cls.calculate_cache_miss_rate(metrics),
            'branch_miss_rate': cls.calculate_branch_miss_rate(metrics),
            'l1d_miss_rate': cls.calculate_l1d_miss_rate(metrics),
        }


class Formatter:
    """Format performance comparison output."""

    @staticmethod
    def format_number(value: Optional[float], unit: str = '') -> str:
        """Format a number with appropriate units and precision."""
        if value is None:
            return 'N/A'

        if unit == 'billions':
            return f"{value / 1e9:.2f}B"
        elif unit == 'millions':
            return f"{value / 1e6:.2f}M"
        elif unit == 'percent':
            return f"{value:.1f}%"
        elif unit == 'seconds':
            return f"{value:.3f}s"
        elif unit == 'ratio':
            return f"{value:.2f}"
        else:
            return f"{value:.0f}"

    @staticmethod
    def format_delta(val1: Optional[float], val2: Optional[float]) -> str:
        """Format delta between two values as ratio or percentage."""
        if val1 is None or val2 is None or val1 == 0:
            return 'N/A'

        ratio = val2 / val1
        pct_change = ((val2 - val1) / val1) * 100

        if ratio > 1:
            return f"{ratio:.2f}x"
        else:
            return f"{pct_change:+.0f}%"

    @staticmethod
    def print_table(label1: str, metrics1: Dict, label2: str, metrics2: Dict,
                    derived1: Dict, derived2: Dict) -> None:
        """Print formatted comparison table."""

        print("\n" + "=" * 85)
        print(f"Performance Comparison: {label1} vs {label2}")
        print("=" * 85)
        print()

        # Table header
        print(f"{'Metric':<30} {label1:>18} {label2:>18} {'Delta':>15}")
        print("-" * 85)

        # Raw metrics
        rows = [
            ("Runtime (seconds)", metrics1.get('runtime'), metrics2.get('runtime'), 'seconds'),
            ("Cycles", metrics1.get('cycles'), metrics2.get('cycles'), 'billions'),
            ("Instructions", metrics1.get('instructions'), metrics2.get('instructions'), 'billions'),
            ("Cache References", metrics1.get('cache_references'), metrics2.get('cache_references'), 'millions'),
            ("Cache Misses", metrics1.get('cache_misses'), metrics2.get('cache_misses'), 'millions'),
            ("L1 D-cache Loads", metrics1.get('l1d_loads'), metrics2.get('l1d_loads'), 'millions'),
            ("L1 D-cache Load Misses", metrics1.get('l1d_load_misses'), metrics2.get('l1d_load_misses'), 'millions'),
            ("Branches", metrics1.get('branches'), metrics2.get('branches'), 'millions'),
            ("Branch Misses", metrics1.get('branch_misses'), metrics2.get('branch_misses'), 'millions'),
        ]

        for metric_name, val1, val2, unit in rows:
            fmt1 = Formatter.format_number(val1, unit)
            fmt2 = Formatter.format_number(val2, unit)
            delta = Formatter.format_delta(val1, val2)
            print(f"{metric_name:<30} {fmt1:>18} {fmt2:>18} {delta:>15}")

        print()
        print("-" * 85)
        print("Derived Metrics")
        print("-" * 85)

        # Derived metrics
        derived_rows = [
            ("IPC (Instr/Cycle)", derived1.get('ipc'), derived2.get('ipc'), 'ratio'),
            ("Cache Miss Rate", derived1.get('cache_miss_rate'), derived2.get('cache_miss_rate'), 'percent'),
            ("L1 Miss Rate", derived1.get('l1d_miss_rate'), derived2.get('l1d_miss_rate'), 'percent'),
            ("Branch Miss Rate", derived1.get('branch_miss_rate'), derived2.get('branch_miss_rate'), 'percent'),
        ]

        for metric_name, val1, val2, unit in derived_rows:
            fmt1 = Formatter.format_number(val1, unit)
            fmt2 = Formatter.format_number(val2, unit)
            delta = Formatter.format_delta(val1, val2)
            print(f"{metric_name:<30} {fmt1:>18} {fmt2:>18} {delta:>15}")

        print()
        print("=" * 85)

        # Interpretation
        print("\nInterpretation:")
        print("-" * 85)

        if metrics1.get('cycles') and metrics2.get('cycles'):
            slowdown = metrics2['cycles'] / metrics1['cycles']
            print(f"  • Pointer chasing is {slowdown:.1f}x slower (more cycles)")

        if derived1.get('ipc') and derived2.get('ipc'):
            ipc_loss = (1 - (derived2['ipc'] / derived1['ipc'])) * 100
            print(f"  • IPC dropped by {ipc_loss:.0f}% (instruction parallelism reduced)")

        if derived1.get('cache_miss_rate') and derived2.get('cache_miss_rate'):
            miss_increase = derived2['cache_miss_rate'] / derived1['cache_miss_rate']
            print(f"  • Cache miss rate increased {miss_increase:.1f}x")
            print(f"    Sequential: {derived1['cache_miss_rate']:.1f}% vs Pointer: {derived2['cache_miss_rate']:.1f}%")

        if derived1.get('l1d_miss_rate') and derived2.get('l1d_miss_rate'):
            l1_increase = derived2['l1d_miss_rate'] / derived1['l1d_miss_rate']
            print(f"  • L1 D-cache miss rate increased {l1_increase:.1f}x (prefetching defeated)")
            print(f"    Sequential: {derived1['l1d_miss_rate']:.1f}% vs Pointer: {derived2['l1d_miss_rate']:.1f}%")

        print()


def main():
    """Main entry point."""
    if len(sys.argv) < 3:
        print("Usage: python perf_compare.py <file1> <file2> [--labels 'Label 1' 'Label 2']")
        print()
        print("Example:")
        print("  python perf_compare.py perf_seq.txt perf_ptr.txt")
        print("  python perf_compare.py perf_seq.txt perf_ptr.txt --labels 'Sequential' 'Pointer'")
        sys.exit(1)

    file1 = sys.argv[1]
    file2 = sys.argv[2]

    # Parse optional labels
    label1 = Path(file1).stem
    label2 = Path(file2).stem

    if '--labels' in sys.argv:
        try:
            idx = sys.argv.index('--labels')
            label1 = sys.argv[idx + 1]
            label2 = sys.argv[idx + 2]
        except (IndexError, ValueError):
            print("Error: --labels requires two arguments")
            sys.exit(1)

    # Parse perf stat files
    print(f"Parsing {file1}...")
    metrics1 = PerfStatParser.parse_file(file1)

    print(f"Parsing {file2}...")
    metrics2 = PerfStatParser.parse_file(file2)

    # Compute derived metrics
    derived1 = MetricsCalculator.compute_all(metrics1)
    derived2 = MetricsCalculator.compute_all(metrics2)

    # Print comparison table
    Formatter.print_table(label1, metrics1, label2, metrics2, derived1, derived2)


if __name__ == '__main__':
    main()
