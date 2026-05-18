---
applyTo: "**"
---

# Harness 엔지니어링 실습 리포지토리 가이드

## 1. 목표 & 배경
- 하네스 엔지니어링 실습은 OctoFit Tracker와 비슷한 backend/frontend 스택을 대상으로 Copilot agent mode로 테스트/검증 파이프라인을 만들고, 이를 통해 자동화된 테스트 하네스를 이해하는 것이 목표입니다.
- 참여자는 이 가이드와 함께 Codespaces에서 일관된 환경을 띄우고, Copilot agent mode에게 명확한 프롬프트(이 문서와 `.github/prompts/harness_engineering.prompt.md` 참조)를 전달하여 작업을 순차적으로 진행합니다.

## 2. 환경 요구
- Codespace: `Create Codespace`를 눌러 devcontainer을 기반으로 Python 3.11 + Node 18 + MongoDB 클러스터 구성이 미리 설치된 환경을 띄웁니다.
- Devcontainer 세팅에 `forwardPorts`로 8000(Django), 3000(React), 27017(Mongo)을 명시하고, `postCreateCommand`에서 `pip install -r backend/requirements.txt`와 `npm install frontend`를 실행합니다.
- `backend/venv` 가상환경을 만들고, front/backend 명령은 항상 `python backend/manage.py ...`, `npm --prefix frontend ...` 같은 절대 경로 방식으로 실행합니다.

## 3. 작업 흐름
1. Codespace를 생성하고 Copilot Chat을 연 다음, `build-octofit-app` 브랜치에서 시작합니다.
2. 하네스 구조(backend 하위 Django 앱, frontend 테스트 페이지, mock server) 확인 후 Copilot agent mode에게 `harness_engineering` prompt를 전달하여 구조/파일을 생성하도록 합니다.
3. Agent가 테스트 하네스 코드를 만들 때마다 `npm test`/`python backend/manage.py test`를 실행하여 로그를 확인하고, 로그/스크린샷을 `artifacts/`와 Markdown 노트에 기록합니다.
4. 작업 결과는 반드시 Git에 커밋한 뒤 `git push origin build-octofit-app`으로 업로드합니다. Copilot이 자동으로 커밋/푸시하도록 지시하는 것도 허용하며, 수행 로그를 확인하세요.

## 4. Agent 제약 & 검증 포인트
- Agent mode는 디렉터리를 절대 변경하면 안 되며 명령어에 경로를 명시해야 합니다.
- 테스트 하네스 코드 작성 중에는 `backend/tests/` 아래에 unittest 또는 pytest 스타일 파일을 생성하고, `frontend/tests/`에서도 React 테스트를 추가할 수 있습니다.
- `pip install` 등의 명령은 CLI 출력과 함께 `pip freeze` 결과를 복사해서 문서에 붙여 넣게 하며, `npm test` 또는 `python -m pytest` 결과를 슬라이드/앨범에 스크린샷으로 기록합니다.

## 5. 포트 & 보안
- 공개 포트: 8000(Django), 3000(React). Codespace Port Forwarding 탭에서 공개로 설정 가능.
- 비공개 포트: 27017(MongoDB). 외부 공개 금지, Codespaces 탭에서 only private.
- 새로운 포트가 필요하면 사전에 팀에게 승인받고 `devcontainer.json`에 추가하세요.

## 6. 참조
- `.github/prompts/harness_engineering.prompt.md`
- `.github/instructions/octofit_tracker_setup_project.instructions.md`
- `docs/octofit_story.md`
- `scripts/ppt_hook.sh` (새로운 노트/슬라이드 요약을 만들어야 할 때 사용)
