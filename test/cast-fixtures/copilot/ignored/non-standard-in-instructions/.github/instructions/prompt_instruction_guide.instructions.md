---
applyTo: "**"
---

# Prompt 작성 지침(프롬프트 인스트럭션)

이 문서는 Copilot agent mode 기반 실습을 구성하는 프롬프트를 만들 때 참고할 가이드입니다. `hooks`에서 자동으로 프롬프트를 추가하는 플로우에 사용할 수 있도록 형식을 준수하세요.

## 1. 목적
- 실습 참가자가 Copilot에게 무엇을 요청해야 하는지 명확하게 안내합니다.
- 에이전트에게 전달할 명령어는 행동 중심으로 작성하고, 기대 결과 및 제약 조건을 함께 담습니다.

## 2. 구성 템플릿
1. **상황/컨텍스트**: 현재 리포지토리/브랜치와 목표를 간단히 설명합니다.
2. **행동 요청**: Copilot에게 구체적으로 어떤 작업(파일 생성/수정, 브랜치 생성 등)을 수행할지 친절한 문장으로 요청합니다.
3. **제약 조건**: 사용할 브랜치명, 포트, 디렉터리 변경 금지 등 실습 규칙을 덧붙입니다.
4. **확인 항목**: Copilot 실행 후 참가자가 검토해야 할 결과(예: 브랜치가 GitHub에 업로드되었는지, 테스트 통과 여부 등)를 요약합니다.

예:
```prompt
현재 `build-octofit-app` 브랜치에서 OctoFit 앱 백엔드에 `health_metrics` 모델을 추가하는 Django 마이그레이션 코드와 테스트를 작성해 주세요.
- `backend/octofit_tracker/models.py`와 `backend/octofit_tracker/tests.py`를 수정합니다.
- `python backend/manage.py makemigrations` → `python backend/manage.py migrate` 명령은 숫자만 포함된 버전 없이 실행합니다.
- `pipenv` 대신 `venv` 환경을 사용하며, 디렉터리는 이동하지 않습니다.
- Copilot이 생성한 테스트 결과와 `git status` 출력도 캡쳐해서 보여 주세요.
```

## 3. Hook과 연계할 때 주의
- 훅은 새로운 프롬프트 텍스트 파일을 넘겨받아 슬라이드나 instructions에 자동 삽입합니다. `scripts/ppt_hook.sh`와 같이 `cat`으로 이어붙이는 훅은 특별한 포맷 없이 실행되지만, 프롬프트 본문은 위 템플릿을 따라야 이후에 복제/변형도 편합니다.
- 프롬프트를 `.github/prompts`에 저장할 경우 파일명에 `_prompt` 접미사와 `.prompt.md` 확장자를 사용하세요.
- 훅 스크립트를 실행할 때는 반드시 현재 브랜치가 실습 브랜치인지 확인한 뒤 `git status`가 깔끔한 상태인지 체크합니다.

## 4. 참고
- 실습에서 사용한 Copilot 프롬프트는 `.github/prompts/github_copilot_training.prompt.md`와 `.github/prompts/*` 아래에 모아서 관리하세요.
- `skills-build-applications-w-copilot-agent-mode`의 `.github/steps/1-preparing.md`에 있는 Agent 브랜치 생성 프롬프트도 큰 도움이 됩니다.
