-- Add exam grading permission
INSERT INTO permissions (code, resource, action, description) VALUES
('exam.grade', 'exam', 'grade', 'Grade essay answers manually')
ON CONFLICT (code) DO NOTHING;

-- Grant to admin and panitia roles
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name IN ('superadmin', 'admin', 'panitia') AND p.code = 'exam.grade'
ON CONFLICT DO NOTHING;
