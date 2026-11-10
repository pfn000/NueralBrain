import init, { 
            init_brain, process_input, tick_brain, 
            trigger_reward, trigger_discipline,
            enter_sleep, wake_up, get_brain_state 
        } from './pkg/bionetic_brain.js';
        
        let animFrame;
        
        async function boot() {
            await init();
            init_brain();
            
            // Register service worker for background dreaming
            if ('serviceWorker' in navigator) {
                const reg = await navigator.serviceWorker.register('/sw.js');
                console.log('Dream engine registered');
            }
            
            document.getElementById('output').textContent += 
                'Brain online. 512 dots across 8 corpora.\n' +
                'Trans-binary channels established.\n' +
                'Awaiting input...\n\n';
            
            // Start tick loop
            function loop() {
                const status = JSON.parse(tick_brain());
                
                document.getElementById('s-dots').textContent = status.dots || 0;
                document.getElementById('s-pulses').textContent = status.pulses || 0;
                document.getElementById('s-clumps').textContent = status.clumps || 0;
                document.getElementById('s-state').textContent = status.state || '?';
                document.getElementById('s-tick').textContent = status.tick || 0;
                
                animFrame = requestAnimationFrame(loop);
            }
            loop();
            
            // Save state periodically for dream engine
            setInterval(() => {
                const state = get_brain_state();
                if (navigator.serviceWorker.controller) {
                    navigator.serviceWorker.controller.postMessage({
                        type: 'SAVE_STATE',
                        data: state
                    });
                }
            }, 10000);
        }
        
        window.sendInput = function() {
            const input = document.getElementById('input');
            const output = document.getElementById('output');
            const text = input.value;
            
            output.textContent += `> ${text}\n`;
            
            const result = process_input(text);
            output.textContent += `◉ ${result}\n\n`;
            output.scrollTop = output.scrollHeight;
            
            input.value = '';
        };
        
        window.triggerReward = function(mag) {
            trigger_reward(mag, 'manual_reward');
            document.getElementById('output').textContent += 
                `⊕ Reward signal (${mag}) applied\n`;
        };
        
        window.triggerDiscipline = function(mag) {
            trigger_discipline(mag, 'manual_discipline');
            document.getElementById('output').textContent += 
                `⊖ Discipline signal (${mag}) applied\n`;
        };
        
        window.enterSleep = function() {
            enter_sleep();
            document.getElementById('output').textContent += 
                `◑ Entering dream state...\n`;
        };
        
        window.wakeUp = function() {
            wake_up();
            document.getElementById('output').textContent += 
                `◉ Waking up. Consolidation complete.\n`;
        };
        
        // Handle tab close — notify service worker to keep dreaming
        window.addEventListener('beforeunload', () => {
            if (navigator.serviceWorker.controller) {
                const state = get_brain_state();
                navigator.serviceWorker.controller.postMessage({
                    type: 'SAVE_STATE',
                    data: state
                });
                navigator.serviceWorker.controller.postMessage({
                    type: 'SLEEP'
                });
            }
        });
        
        document.getElementById('input').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') sendInput();
        });
        
        boot();
