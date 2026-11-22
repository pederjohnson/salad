-- Fees are in cents (USD).
INSERT INTO terms (semester, normal_fee_cents, effective_date, late_fee_cents, due_date) VALUES
('Fall 2023', 1000, '2023-05-01', NULL, NULL),
('Spring 2024', 1000, '2024-01-01', NULL, NULL),
('Fall 2024', 1000, '2024-05-01', NULL, NULL),
('Spring 2025', 1500, '2025-01-01', NULL, NULL),
('Fall 2025', 1500, '2025-05-01', 2000, '2025-10-6'),
('Spring 2026', 1500, '2026-01-01', 2000, '2026-2-16'),
('Fall 2026', 1500, '2026-05-01', 2000, '2026-10-5'),
('Spring 2027', 2000, '2027-01-01', 2500, '2027-2-15');
