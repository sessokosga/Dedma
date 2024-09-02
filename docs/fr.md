Dedma v0.1.2
Générateur de notes de versions (Release notes)
Un outil qui converti vos derniers commit en notes de version.

La note est générée en anglais par défaut.

Pour générer une note en français, ajouter `LANG_FR` dans les variables d'environnement.
Pour les systèmes basés sur Unix
	LANG_FR=1 

Pour Windows PowerShell
	$Env:LANG_FR=1

Générer des notes à partir des commit dans un fichier
    dedma fichier_d_entree fichier_de_sortie

Générer les notes à partir des commit de Git
    dedma fichier_de_sortie
    
ou 

    dedma

pour générer les notes dans le fichier `whats_new.md`

Structure de commit idéale
    type (titre): contenu

Le `titre` et le `contenu` peuvent être ce que vous voulez.

Voici une liste des types supportés actuellement, classés par ordre d'apparence dans la note générées.

 ______________________________________
 
|   type   | nom complet               |
| -------- | ------------------------- |
|   feat   | Nouvelles fonctionnalités |
|   fix    | Correction d'erreur       |
|  chore   | Chore                     |
| refactor | Refactoring               |
|   docs   | Documentation             |
|  style   | Style de Code             |
|   test   | Test                      |
|   perf   | Performances              |
|    ci    | Déploiements Continue     |
|  build   | Système de Build          |
|  revert  | Annulations               |
|  update  | Mise à jour               |
 
 --------------------------------------