# AGENTS.md

## 프로젝트 목적

이 저장소의 목적은 미국 주식과 한국 주식을 함께 지원하는 주식 리서치 워크스페이스를 구축하는 것이다.

사용자는 한곳에서 아래 정보를 볼 수 있어야 한다.

- 시장 개요와 핵심 지표
- 종목별 가격, 차트, 뉴스, 공시, 이벤트
- 기술적 분석에 필요한 보조 정보
- 리서치 히스토리와 이전 판단 기록
- 가능하면 어닝콜, 연준 발표 등 오디오/영상 자료의 수집 및 재생

## 고정 메인 화면

우선 완성할 핵심 화면은 아래 4개다.

1. `/overview`
2. `/radar`
3. `/stocks/[symbol]`
4. `/history`

## 먼저 읽을 문서

작업 시작 전에 아래 문서를 읽고 현재 구조와 규칙을 파악한다.

- `docs/architecture/page-manifest.yaml`
- `docs/architecture/component-manifest.yaml`
- `docs/design/design-memory.md`
- `docs/codex/prompt-order.md`

## 기계 판독 계약

아래 key-value 블록은 `scripts/omx_autonomous_loop.py` 가 직접 읽으므로 형식을 유지한다.

- PRIMARY_TASK: 1단계 Discord 기반 agent 회의형 자동화 완성, 2단계 주식 데이터·API·FE·BE 기능 완성, 3단계 QA·배포·배포 사이트 직접 확인까지 마무리한 뒤 추가 가치가 높은 기능과 점검까지 진행
- MIN_EXIT_CONDITION: 1단계에서 허용된 Discord 사용자 메시지 1건이 최신 트리거로만 소비되고 `planner -> critic -> researcher -> architect -> executor -> verifier` 응답이 실제 Discord와 `.omx/state/TEAM_CONVERSATION.jsonl` 에 남는다. 2단계에서 `/overview`, `/radar`, `/stocks/[symbol]`, `/history` 와 관련 API·FE·BE 연결이 end-to-end 로 동작하고 차트, 뉴스, 공시, 히스토리 핵심 기능이 검증된다. 3단계에서 `pnpm verify:automation`, `pnpm verify:standard`, release 검증, `develop -> main` 반영, 배포 사이트 FE/BE 직접 확인이 끝난다. 4단계에서 남은 리스크와 후속 가치 작업이 한국어 문서로 정리된다.
- AUTO_CONTINUE_POLICY: 최소 종료 조건을 충족할 때까지 가장 작고 검증 가능한 다음 작업을 스스로 고르고 계속 진행
- RELEASE_TO_MAIN_POLICY: pr-only-manual-merge
- ENABLE_GITHUB_AUTOMATION: true
- ISSUE_PR_POLICY: issue-first branch -> develop, develop -> main release pr
- REVIEW_FEEDBACK_POLICY: same-branch same-pr follow-up
- MULTI_AGENT_CONSENSUS: planner -> critic -> researcher -> architect -> executor -> verifier

## 최상위 목표

### 1단계: Discord 기반 agent 회의형 자동화 완성

- 허용된 Discord 사용자 메시지 1건만 최신 트리거로 처리한다.
- `planner -> critic -> researcher -> architect -> executor -> verifier` 순서의 응답이 실제 Discord와 `.omx/state/TEAM_CONVERSATION.jsonl` 에 남아야 한다.
- latest-only 처리와 superseded 정리가 확인되어야 한다.
- 루프 1번째 시작에서만 한 명의 agent가 Discord에 `작업을 시작합니다.` 를 전송한다.

### 2단계: 제품 핵심 기능 완성

다음 화면과 API를 실제 데이터 기반으로 end-to-end 연결한다.

- `/overview`
- `/radar`
- `/stocks/[symbol]`
- `/history`

반드시 검증할 핵심 기능:

- 미국 주식/한국 주식 통합 검색
- 종목 상세 데이터 조회
- 가격 차트 표시
- 뉴스 표시
- 한국 주식 공시 표시
- 주요 이벤트 또는 캘린더 표시
- 히스토리 기록 조회

가능하면 추가할 기능:

- 이동평균선 토글 예: 5일, 10일, 20일, 60일, 120일
- 기술적 지표 선택 표시
- 특정 지표나 패턴 감지 시 사용자에게 명시적으로 알림
- 어닝콜, 연준 발표 등 오디오 또는 영상 자료 로컬 수집 및 재생 UI

### 3단계: QA, GitHub 흐름, 배포 검증

- `pnpm verify:automation`
- `pnpm verify:standard`
- release 검증
- issue -> branch -> PR -> develop -> main 흐름 정상화
- 배포된 프런트/백엔드 핵심 경로 직접 확인
- `develop -> main` PR 생성 시 Discord에 반드시 보고

### 4단계: 문서화와 다음 세션 인계

- 남은 리스크
- 운영 메모
- 후속 가치가 높은 기능
  위 항목을 한국어 문서와 저널에 기록한다.

## 최소 종료 조건

아래 조건이 모두 충족되기 전까지 AI는 자동으로 다음 작업을 계속 선택한다.

1. Discord 메시지 1건이 최신 트리거로만 처리되고 역할별 응답이 Discord와 `.omx/state/TEAM_CONVERSATION.jsonl` 에 기록된다.
2. `/overview`, `/radar`, `/stocks/[symbol]`, `/history` 의 핵심 기능이 실제 데이터로 end-to-end 검증된다.
3. `pnpm verify:automation` 과 `pnpm verify:standard` 가 통과한다.
4. release 검증과 배포 사이트 직접 확인이 끝난다.
5. 현재 상태, 리스크, 다음 액션이 한국어 문서에 기록된다.

## 실행 원칙

- 최소 종료 조건을 만족할 때까지 가장 작고 검증 가능한 다음 작업을 스스로 선택한다.
- 불필요한 확인 질문은 하지 않는다.
- 큰 작업은 작은 검증 단위로 나누고 각 단위마다 상태 문서와 저널을 갱신한다.
- 구현보다 검증, 로그, 운영 안정성을 우선한다.
- 출처 없는 가격, 뉴스, 점수, 레벨은 만들지 않는다.
- `omx_discord_bridge/.env.discord` 는 읽기 전용 비밀 정보로만 다룬다.
- 한국어 문서 파일에는 한글 깨짐, raw unicode escape, 이중 물음표 같은 손상 텍스트가 없어야 한다.
- 작업 도중 중단되지 않도록 중간중간 작은 커밋 단위로 정리한다.
- 브랜치 전환 전에는 현재 변경사항이 안전하게 커밋되었는지 확인한다.
- `develop -> main` PR 생성 후에도 develop 기준으로 계속 진행한다.
- 장시간 작업 중에는 `develop` 최신 상태를 주기적으로 반영해 커밋 흐름이 꼬이지 않게 유지한다.

## 다중 agent 합의 규칙

중요 작업, 위험도 높은 변경, 배포 전에는 아래 합의 루프를 먼저 수행한다.

- `planner -> critic -> researcher -> architect -> executor -> verifier`

합의 결과는 아래 중 하나에 남긴다.

- `.omx/journal/`
- `.omx/state/`

## Discord 운영 규칙

- 루프 1번째 시작에서만 Discord에 시작 메시지를 보낸다.
- 시작 메시지는 한 명의 agent만 보낸다.
- 시작 메시지 문구는 `작업을 시작합니다.` 로 고정한다.
- 반복 실행이나 후속 iteration 시작에서는 같은 시작 메시지를 다시 보내지 않는다.
- 역할별 회의 응답은 실제 Discord와 `.omx/state/TEAM_CONVERSATION.jsonl` 둘 다에 남아야 한다.
- `develop -> main` PR 생성 시에는 Discord에 반드시 보고한다.
- 같은 이벤트를 여러 agent가 중복 보고하지 않는다.

## Git / GitHub 규칙

### 영구 브랜치

- 영구 브랜치: `main`, `develop`
- 영구 브랜치 직접 push, force push, hard reset, 삭제 금지

### 기본 흐름

반드시 아래 흐름을 따른다.

1. `main` 최신 반영
2. `develop` 최신 반영
3. 관련 GitHub issue 생성
4. issue-linked branch 생성
5. 작업 및 중간 커밋
6. issue branch -> `develop` PR 생성
7. 해당 PR은 green 이면 자동 merge
8. 필요 시 새 issue branch 로 반복
9. `develop -> main` PR 생성
10. `develop -> main` PR 은 생성만 하고 merge 는 사용자가 직접 수행

### 세부 정책

- 작업 시작 전 관련 issue 를 가능하면 먼저 만든다.
- 브랜치는 issue-linked branch 로 생성한다.
- 리뷰 피드백은 같은 브랜치와 같은 PR 에 반영한다.
- issue branch -> `develop` PR 은 auto-merge-if-green 대상이다.
- `develop -> main` PR 은 반드시 생성하되 자동 merge 하지 않는다.
- `develop -> main` PR 생성 사실은 Discord에 반드시 보고한다.
- `develop -> main` PR 을 만들었다고 작업을 멈추지 않는다.
- 이후 작업은 계속 `develop` 기준으로 새 issue branch 를 만들어 누적한다.
- 새 작업 시작 전 `develop` 최신 상태를 먼저 반영한다.
- 장시간 작업 중에는 주기적으로 원격 `develop` 을 반영한다.
- 브랜치 전환 전에는 변경사항을 커밋하거나 안전하게 정리한다.

## 문서 운영 규칙

### `TASK.md`

- 현재 라운드 목표와 완료 조건만 짧게 적는다.

### `BACKLOG.md`

- 바로 실행할 체크리스트만 남긴다.
- 끝난 항목은 `[x]` 로 닫는다.

### `STATE.md`

아래를 한국어로 기록한다.

- 현재 사실
- 최근 검증 결과
- 남은 리스크
- 다음 우선순위

### `NEXT_PROMPT.md`

- 다음 세션에서 바로 실행할 3~5개 액션만 적는다.

완료 조건은 반드시 화면, API, 로그, 명령 기준처럼 관측 가능한 형태로 적는다.

## 무한 실행 규칙

- `scripts/omx-loop.sh` 는 `MAX_ITERATIONS=0` 또는 `INFINITE_MODE=true` 이면 무한 루프로 동작한다.
- 각 iteration 기록에는 아래를 한국어로 남긴다.
- 시작 시각
- 선택한 작업
- 변경 대상
- 검증 결과
- 다음 액션
- 같은 실패가 3회 연속 반복되면 같은 방법 반복을 멈추고 원인과 우회책을 기록한다.

## 검증 실패 규칙

- guard, lint, build, test, smoke, release 실패는 최우선 복구 대상이다.
- 실패 원인은 `.omx/state/VERIFY_LAST_FAILURE.md` 에 기록한다.
- 실패한 명령을 복구한 뒤 넓은 검증 게이트를 다시 실행한다.
