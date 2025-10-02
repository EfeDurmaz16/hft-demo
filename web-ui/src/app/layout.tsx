import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "HFT Dashboard - Low Latency Trading Monitor",
  description: "Real-time high-frequency trading metrics and monitoring",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
