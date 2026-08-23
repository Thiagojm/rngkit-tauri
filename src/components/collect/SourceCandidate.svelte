<script lang="ts">
  import { candidateLabel } from '../../state/controls';
  import type { SourceCandidateView } from '../../state/types';

  let {
    candidate,
    selected,
    disabled,
    onSelect,
  }: {
    candidate: SourceCandidateView;
    selected: boolean;
    disabled: boolean;
    onSelect: (token: string) => void;
  } = $props();
</script>

<label class="flex items-center gap-2 text-sm">
  <input
    type="radio"
    name="entropy-source"
    value={candidate.token}
    checked={selected}
    {disabled}
    onchange={() => onSelect(candidate.token)}
  />
  {candidateLabel(candidate)}
</label>
