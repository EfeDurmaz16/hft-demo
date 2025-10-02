"use client";

import { useEffect, useState } from "react";
import {
  LineChart,
  Line,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";

interface MetricsSnapshot {
  ticks_received: number;
  orders_placed: number;
  latency_p50: number;
  latency_p99: number;
  latency_mean: number;
  timestamp: number;
}

interface LatencyDataPoint {
  time: string;
  p50: number;
  p99: number;
  mean: number;
}

export default function Dashboard() {
  const [metrics, setMetrics] = useState<MetricsSnapshot | null>(null);
  const [latencyHistory, setLatencyHistory] = useState<LatencyDataPoint[]>([]);
  const [connectionStatus, setConnectionStatus] = useState<string>("Connecting...");

  useEffect(() => {
    const ws = new WebSocket("ws://localhost:9090/ws");

    ws.onopen = () => {
      setConnectionStatus("Connected");
      console.log("WebSocket connected");
    };

    ws.onmessage = (event) => {
      try {
        const snapshot: MetricsSnapshot = JSON.parse(event.data);
        setMetrics(snapshot);

        // Add to history
        const time = new Date(snapshot.timestamp * 1000).toLocaleTimeString();
        setLatencyHistory((prev) => {
          const newHistory = [
            ...prev,
            {
              time,
              p50: snapshot.latency_p50,
              p99: snapshot.latency_p99,
              mean: snapshot.latency_mean,
            },
          ];
          // Keep last 30 data points
          return newHistory.slice(-30);
        });
      } catch (err) {
        console.error("Failed to parse metrics:", err);
      }
    };

    ws.onerror = (error) => {
      setConnectionStatus("Error");
      console.error("WebSocket error:", error);
    };

    ws.onclose = () => {
      setConnectionStatus("Disconnected");
      console.log("WebSocket disconnected");
    };

    return () => {
      ws.close();
    };
  }, []);

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100 p-8">
      <div className="max-w-7xl mx-auto">
        <header className="mb-8">
          <h1 className="text-4xl font-bold mb-2">HFT Trading Dashboard</h1>
          <p className="text-gray-400">
            Real-time low-latency trading metrics • Status:{" "}
            <span
              className={
                connectionStatus === "Connected"
                  ? "text-green-400"
                  : "text-red-400"
              }
            >
              {connectionStatus}
            </span>
          </p>
        </header>

        {/* Metrics Cards */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <MetricCard
            title="Ticks Received"
            value={metrics?.ticks_received.toLocaleString() || "0"}
            unit="total"
          />
          <MetricCard
            title="Orders Placed"
            value={metrics?.orders_placed.toLocaleString() || "0"}
            unit="total"
          />
          <MetricCard
            title="P50 Latency"
            value={metrics?.latency_p50.toFixed(2) || "0"}
            unit="µs"
          />
          <MetricCard
            title="P99 Latency"
            value={metrics?.latency_p99.toFixed(2) || "0"}
            unit="µs"
          />
        </div>

        {/* Charts */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Latency Line Chart */}
          <div className="bg-gray-900 rounded-lg p-6">
            <h2 className="text-xl font-semibold mb-4">
              Latency Over Time (µs)
            </h2>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={latencyHistory}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="time" stroke="#9CA3AF" />
                <YAxis stroke="#9CA3AF" />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#1F2937",
                    border: "1px solid #374151",
                  }}
                />
                <Legend />
                <Line
                  type="monotone"
                  dataKey="p50"
                  stroke="#10B981"
                  name="P50"
                  dot={false}
                />
                <Line
                  type="monotone"
                  dataKey="p99"
                  stroke="#F59E0B"
                  name="P99"
                  dot={false}
                />
                <Line
                  type="monotone"
                  dataKey="mean"
                  stroke="#3B82F6"
                  name="Mean"
                  dot={false}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>

          {/* Latency Distribution */}
          <div className="bg-gray-900 rounded-lg p-6">
            <h2 className="text-xl font-semibold mb-4">
              Current Latency Distribution
            </h2>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart
                data={[
                  { name: "P50", value: metrics?.latency_p50 || 0 },
                  { name: "Mean", value: metrics?.latency_mean || 0 },
                  { name: "P99", value: metrics?.latency_p99 || 0 },
                ]}
              >
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="name" stroke="#9CA3AF" />
                <YAxis stroke="#9CA3AF" />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#1F2937",
                    border: "1px solid #374151",
                  }}
                />
                <Bar dataKey="value" fill="#8B5CF6" />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* System Info */}
        <div className="mt-8 bg-gray-900 rounded-lg p-6">
          <h2 className="text-xl font-semibold mb-4">System Information</h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
            <div>
              <p className="text-gray-400">Tick Rate</p>
              <p className="text-lg font-semibold">10,000 /sec</p>
            </div>
            <div>
              <p className="text-gray-400">Transport</p>
              <p className="text-lg font-semibold">UDP</p>
            </div>
            <div>
              <p className="text-gray-400">Strategy</p>
              <p className="text-lg font-semibold">Threshold-based</p>
            </div>
            <div>
              <p className="text-gray-400">Target Latency</p>
              <p className="text-lg font-semibold">&lt; 100µs</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function MetricCard({
  title,
  value,
  unit,
}: {
  title: string;
  value: string;
  unit: string;
}) {
  return (
    <div className="bg-gray-900 rounded-lg p-6">
      <h3 className="text-sm text-gray-400 mb-2">{title}</h3>
      <p className="text-3xl font-bold">
        {value}
        <span className="text-lg text-gray-500 ml-2">{unit}</span>
      </p>
    </div>
  );
}
