-- logical: 0025_repo_activity_branch_rename — allow branch_rename push_type

ALTER TABLE repository_activity
  DROP CONSTRAINT IF EXISTS repository_activity_push_type_check;

ALTER TABLE repository_activity
  ADD CONSTRAINT repository_activity_push_type_check
  CHECK (push_type IN (
    'push', 'force_push', 'pr_merge', 'branch_creation', 'branch_deletion', 'branch_rename'
  ));
