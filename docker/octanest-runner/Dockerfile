# Official Octanest Actions runner (ACT-04 / D-ACT-11).
# Based on Gitea act_runner — executes jobs outside the API process (ACT-07).
# Pin: gitea/act_runner:0.2.11 (bump intentionally; document in README).
FROM gitea/act_runner:0.2.11

LABEL org.opencontainers.image.title="octanest-runner" \
      org.opencontainers.image.description="Official Octanest Actions runner (act_runner lineage)" \
      org.opencontainers.image.source="https://github.com/octanest/octanest"

# Default labels: ubuntu-latest via docker schema (D-ACT-09 / D-ACT-11).
ENV OCTANEST_RUNNER_LABELS="ubuntu-latest:docker://node:20-bookworm,self-hosted"

COPY entrypoint.sh /usr/local/bin/octanest-runner-entrypoint.sh
RUN chmod +x /usr/local/bin/octanest-runner-entrypoint.sh

ENTRYPOINT ["/usr/local/bin/octanest-runner-entrypoint.sh"]
