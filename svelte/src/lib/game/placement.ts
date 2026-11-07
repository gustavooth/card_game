import type { CardDef, Content, Side, Snapshot } from '#lib/types';
import { ARENA_HEIGHT, ARENA_WIDTH } from './constants';

function enemyPrincessAlive(snapshot: Snapshot, localSide: Side, lane: number): boolean {
	// Entidades: id, kind, side, typeIdx, hp, maxHp, x, y, radius, ...
	return snapshot.e.some(
		(entity) =>
			entity[1] === 2 &&
			entity[2] !== localSide &&
			(lane === 0 ? entity[6] < 9 : entity[6] > 9)
	);
}

export function cardDeploysAnywhere(card: CardDef, content: Content): boolean {
	let skills: { skill: string }[] = [];
	if (card.type.kind === 'troop') {
		skills = content.definitions.troops[card.type.troop]?.skills ?? [];
	} else if (card.type.kind === 'building') {
		skills = content.definitions.buildings[card.type.building]?.skills ?? [];
	} else {
		return true;
	}
	return skills.some((reference) => {
		const skill = content.definitions.skills[reference.skill];
		return skill?.kind === 'deploy_anywhere' || skill?.kind === 'burrow';
	});
}

export function validatePlacement(
	content: Content,
	card: CardDef,
	x: number,
	y: number,
	snapshot: Snapshot | null,
	localSide: Side
): boolean {
	if (x < 0.3 || x > ARENA_WIDTH - 0.3 || y < 0.3 || y > ARENA_HEIGHT - 0.3) {
		return false;
	}
	if (card.type.kind === 'spell' || cardDeploysAnywhere(card, content)) {
		return true;
	}
	if (y > 15.5 && y < 16.5) {
		return false;
	}
	const ownHalf = localSide === 0 ? y >= 16.35 : y <= 15.65;
	if (!ownHalf && snapshot) {
		const lane = x < 9 ? 0 : 1;
		if (enemyPrincessAlive(snapshot, localSide, lane)) {
			return false;
		}
		const limit = localSide === 0 ? 5.2 : ARENA_HEIGHT - 5.2;
		if (localSide === 0 ? y < limit : y > limit) {
			return false;
		}
	}
	if (!snapshot) {
		return ownHalf;
	}
	const radius =
		card.type.kind === 'troop'
			? (content.definitions.troops[card.type.troop]?.radius ?? 0.4)
			: (content.definitions.buildings[card.type.building]?.radius ?? 0.6);
	for (const entity of snapshot.e) {
		if (entity[1] !== 1 && entity[1] !== 2) {
			continue;
		}
		const distance = Math.hypot(entity[6] - x, entity[7] - y);
		if (distance < entity[8] + radius + 0.15) {
			return false;
		}
	}
	return true;
}
