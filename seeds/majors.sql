-- Reference data, safe to load in every environment and to re-run.
-- Add majors here as members join; members.major must match a name below.

INSERT INTO majors VALUES
('CS', 'Computer Science'),
('CE', 'Computer Engineering'),
('SE', 'Software Engineering'),
('EE', 'Electrical Engineering'),
('MECH', 'Mechanical Engineering'),
('BMEN', 'Biomedical Engineering'),
('SYSE', 'Systems Engineering'),
('PHYS', 'Physics'),
('CHEM', 'Chemistry'),
('BIO', 'Biology'),
('BCHM', 'Biochemistry'),
('BCM', 'Molecular and Cell Biology'),
('NSC', 'Neuroscience'),
('BA', 'Business Administration'),
('ACCT', 'Accounting'),
('FIN', 'Finance'),
('ATCM', 'Actuarial Science'),
('GIS', 'Geospacial Information Science'),
('CGS', 'Cognitive Science'),
('UND', 'Undergraduate Studies'),
('CRIM', 'Criminology'),
('PSY', 'Psychology')
ON CONFLICT DO NOTHING;
